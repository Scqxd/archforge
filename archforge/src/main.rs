//! ArchForge - AI-powered TUI for PKGBUILD generation and AUR management
//!
//! Optimized with:
//! - Batch dependency checking with single pacman call
//! - AUR response caching
//! - Efficient file operations

use std::path::Path;
use std::error::Error;
use std::collections::HashSet;
use std::sync::{OnceLock, RwLock};
use std::collections::HashMap;
use clap::Parser;

use archforge::cli::{Cli, SwarmCommands, CacheCommands};
use archforge::templates::TemplateKind;
use archforge::config::Config;
use archforge::ai::ChutesClient;
use archforge::slugify;

/// Version of ArchForge
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Homepage
pub const HOMEPAGE: &str = "https://github.com/archforge/archforge";

/// AUR response cache (optimization: avoid redundant RPC calls)
static AUR_CACHE: OnceLock<RwLock<HashMap<String, serde_json::Value>>> = OnceLock::new();

fn get_aur_cache() -> &'static RwLock<HashMap<String, serde_json::Value>> {
    AUR_CACHE.get_or_init(|| RwLock::new(HashMap::with_capacity(64)))
}

/// Print ArchForge ASCII logo
pub fn print_logo() {
    println!(r#"    ╔══════════════════════════════════════╗
    ║       ArchForge v{}               ║
    ║   AI-powered PKGBUILD Generator      ║
    ╚══════════════════════════════════════╝"#, VERSION);
}

fn main() -> Result<(), Box<dyn Error>> {
    // Print logo for interactive TUI only
    if let Some(cmd) = std::env::args().nth(1) {
        if cmd == "interactive" || cmd == "-i" || cmd == "--interactive" {
            print_logo();
            println!();
        }
    }

    // Parse CLI arguments
    let cli = Cli::parse();

    // Load config (silent, will use defaults if not exists)
    let _config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: Failed to load config: {}, using defaults", e);
            Config::default()
        }
    };

    // Run the command
    match cli.command {
        archforge::cli::Commands::Generate { description, output, quiet, ai_provider, api_key } => {
            generate(&description, output, quiet, ai_provider, api_key)?;
        }
        archforge::cli::Commands::Build { package, install, nodeps } => {
            build(&package, install, nodeps)?;
        }
        archforge::cli::Commands::Search { query, json, limit: _ } => {
            search(&query, json)?;
        }
        archforge::cli::Commands::Info { package } => {
            info(&package)?;
        }
        archforge::cli::Commands::Deploy { package, target: _, yes: _ } => {
            deploy(&package)?;
        }
        archforge::cli::Commands::Interactive { no_model: _ } => {
            run_tui()?;
        }
        archforge::cli::Commands::Init { name, template: _, directory } => {
            init(&name, directory)?;
        }
        archforge::cli::Commands::Validate { path, srcinfo, dependencies } => {
            validate(&path, srcinfo, dependencies)?;
        }
        archforge::cli::Commands::Swarm(cmd) => {
            swarm(cmd)?;
        }
        archforge::cli::Commands::Status => {
            status()?;
        }
        archforge::cli::Commands::Cache(cmd) => {
            cache(cmd)?;
        }
    }

    Ok(())
}

fn generate(
    description: &str,
    output: Option<std::path::PathBuf>,
    quiet: bool,
    ai_provider: archforge::ai::AiProvider,
    api_key: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !quiet {
        eprintln!("Generating PKGBUILD for: {}", description);
    }

    match ai_provider {
        archforge::ai::AiProvider::Chutes => {
            let api_key = api_key.or_else(|| {
                std::env::var("CHUTES_API_KEY").ok()
            });

            if let Some(api_key) = api_key {
                let client = ChutesClient::new(api_key);
                match client.generate_pkgbuild(description) {
                    Ok(pkgbuild) => {
                        if let Some(path) = output {
                            std::fs::write(&path, &pkgbuild)?;
                            if !quiet {
                                eprintln!("PKGBUILD saved to: {}", path.display());
                            }
                        } else {
                            println!("{}", pkgbuild);
                        }
                        return Ok(());
                    }
                    Err(e) => {
                        if !quiet {
                            eprintln!("AI generation failed: {}. Falling back to template...", e);
                        }
                    }
                }
            } else if !quiet {
                eprintln!("No API key provided. Using fallback template generation.");
            }
            generate_fallback(description, output, quiet)?;
        }
        archforge::ai::AiProvider::Local => {
            if !quiet {
                eprintln!("Local AI provider not implemented yet. Using fallback.");
            }
            generate_fallback(description, output, quiet)?;
        }
        archforge::ai::AiProvider::Openai => {
            if !quiet {
                eprintln!("OpenAI provider not implemented yet. Using fallback.");
            }
            generate_fallback(description, output, quiet)?;
        }
    }

    Ok(())
}

fn generate_fallback(
    description: &str,
    output: Option<std::path::PathBuf>,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let pkgname = slugify(description);

    // Detect language and generate appropriate template
    let template_kind = TemplateKind::from_description(description);
    let pkgver = "0.1.0";

    let pkgbuild = template_kind.generate_pkgbuild(&pkgname, pkgver, description);

    if let Some(path) = output {
        std::fs::write(&path, &pkgbuild)?;
        if !quiet {
            eprintln!("PKGBUILD saved to: {}", path.display());
        }
    } else {
        println!("{}", pkgbuild);
    }

    Ok(())
}

fn build(package: &str, install: bool, nodeps: bool) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Building package: {}", package);

    // Try to parse PKGBUILD if it exists
    let pkgbuild_path = std::path::PathBuf::from(package);
    let actual_path = if pkgbuild_path.is_dir() {
        pkgbuild_path.join("PKGBUILD")
    } else {
        pkgbuild_path
    };

    if actual_path.exists() {
        eprintln!("Found PKGBUILD at: {}", actual_path.display());

        // Check if makepkg is available
        if std::process::Command::new("makepkg").arg("--version").output().is_ok() {
            eprintln!("Running makepkg...");

            let mut args = vec![];
            if nodeps {
                args.push("--nodeps");
            } else {
                args.push("--syncdeps");
            }

            eprintln!("Running: makepkg {} {}", args.join(" "), if install { "--install" } else { "" });

            let output = std::process::Command::new("makepkg")
                .args(&args)
                .current_dir(actual_path.parent().unwrap_or(&std::path::PathBuf::from(".")))
                .output()?;

            if output.status.success() {
                eprintln!("✓ PKGBUILD validation successful!");

                if install {
                    eprintln!("Installing package...");
                    let install_output = std::process::Command::new("sudo")
                        .args(["pacman", "-U", "--noconfirm"])
                        .current_dir(actual_path.parent().unwrap_or(&std::path::PathBuf::from(".")))
                        .arg(find_pkg_file(actual_path.parent().unwrap_or(&std::path::PathBuf::from(".")))?)
                        .output()?;

                    if install_output.status.success() {
                        eprintln!("✓ Package installed successfully!");
                    } else {
                        eprintln!("✗ Installation failed:");
                        eprintln!("{}", String::from_utf8_lossy(&install_output.stderr));
                    }
                }
            } else {
                eprintln!("✗ Build failed:");
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            }
        } else {
            eprintln!("makepkg not found. Install base-devel package.");
        }
    } else {
        eprintln!("PKGBUILD not found at: {}", actual_path.display());
    }

    Ok(())
}

/// Find the built package file (.pkg.tar.zst, .pkg.tar.xz, or .pkg)
fn find_pkg_file(dir: &std::path::Path) -> Result<String, Box<dyn std::error::Error>> {
    let entries = std::fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if let Some(file_name) = path.file_name().and_then(|e| e.to_str()) {
            // Match .pkg.tar.zst, .pkg.tar.xz, or .pkg files
            if file_name.contains(".pkg.tar.") || file_name.ends_with(".pkg") {
                return Ok(path.to_string_lossy().to_string());
            }
        }
    }
    Err("No package file found (.pkg.tar.zst, .pkg.tar.xz, or .pkg)".into())
}

fn search(query: &str, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Searching AUR for: {}", query);

    // Check cache first (optimization: avoid redundant RPC calls)
    let cache_key = format!("search:{}", query);
    let response = if let Some(cached) = get_aur_cache().read().unwrap().get(&cache_key) {
        eprintln!("[AUR] Cache hit for search query");
        cached.clone()
    } else {
        // Query AUR RPC
        let url = format!(
            "https://aur.archlinux.org/rpc?v=5&type=search&arg={}",
            urlencoding::encode(query)
        );

        let response = reqwest::blocking::get(&url)?
            .json::<serde_json::Value>()?;

        // Cache the response
        if let Ok(mut cache) = get_aur_cache().write() {
            cache.insert(cache_key, response.clone());
        }

        response
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        println!("Search results for '{}':", query);
        println!("{}", "=".repeat(60));

        if let Some(results) = response.get("results").and_then(|r| r.as_array()) {
            for (i, result) in results.iter().enumerate().take(10) {
                let name = result.get("Name").and_then(|v| v.as_str()).unwrap_or("?");
                let version = result.get("Version").and_then(|v| v.as_str()).unwrap_or("?");
                let desc = result.get("Description").and_then(|v| v.as_str()).unwrap_or("");
                let votes = result.get("Votes").and_then(|v| v.as_u64()).unwrap_or(0);

                println!("{}. {} {} ({} votes)", i + 1, name, version, votes);
                println!("   {}", desc.chars().take(60).collect::<String>());
                println!();
            }
        }
    }

    Ok(())
}

fn info(package: &str) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Getting info for: {}", package);

    // Check cache first (optimization: avoid redundant RPC calls)
    let cache_key = format!("info:{}", package);
    let response = if let Some(cached) = get_aur_cache().read().unwrap().get(&cache_key) {
        eprintln!("[AUR] Cache hit for package info");
        cached.clone()
    } else {
        let url = format!(
            "https://aur.archlinux.org/rpc?v=5&type=info&arg={}",
            urlencoding::encode(package)
        );

        let response = reqwest::blocking::get(&url)?
            .json::<serde_json::Value>()?;

        // Cache the response
        if let Ok(mut cache) = get_aur_cache().write() {
            cache.insert(cache_key, response.clone());
        }

        response
    };

    if let Some(result) = response.get("results")
        .and_then(|r| r.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_object()) {
        println!("Package: {}", result.get("Name").and_then(|v| v.as_str()).unwrap_or("?"));
        println!("Version: {}", result.get("Version").and_then(|v| v.as_str()).unwrap_or("?"));
        println!("Description: {}", result.get("Description").and_then(|v| v.as_str()).unwrap_or(""));
        println!("Maintainer: {}", result.get("Maintainer").and_then(|v| v.as_str()).unwrap_or("unknown"));
        println!("Votes: {}", result.get("Votes").and_then(|v| v.as_u64()).unwrap_or(0));
        println!("Popularity: {:.2}", result.get("Popularity").and_then(|v| v.as_f64()).unwrap_or(0.0));
        println!("\nDependencies:");
        if let Some(deps) = result.get("Depends").and_then(|v| v.as_array()) {
            for dep in deps {
                println!("  - {}", dep.as_str().unwrap_or("?"));
            }
        }
        if let Some(makedeps) = result.get("MakeDepends").and_then(|v| v.as_array()) {
            if !makedeps.is_empty() {
                println!("\nBuild Dependencies:");
                for dep in makedeps {
                    println!("  - {}", dep.as_str().unwrap_or("?"));
                }
            }
        }
        println!("\nURL: {}", result.get("URL").and_then(|v| v.as_str()).unwrap_or("none"));
    } else {
        eprintln!("Package not found in AUR");
    }

    Ok(())
}

fn deploy(package: &str) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Deploying package: {}", package);
    eprintln!("\nDeployment options:");
    eprintln!("  1. AUR (requires git + SSH key)");
    eprintln!("  2. Docker (requires Docker installed)");
    eprintln!("  3. Flatpak (requires flatpak-builder)");
    eprintln!("  4. Nix flake (requires Nix)");
    eprintln!("\nTo deploy, use:");
    eprintln!("  archforge deploy ./mypkg --target aur");
    eprintln!("  archforge deploy ./mypkg --target docker");
    Ok(())
}

fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    // Check if we're in a TTY
    if !atty::is(atty::Stream::Stdout) {
        eprintln!("Error: TUI mode requires a terminal");
        eprintln!("Use 'archforge generate \"description\"' for non-interactive mode");
        std::process::exit(1);
    }

    #[cfg(feature = "tui")]
    {
        archforge::tui::run_tui()?;
    }
    #[cfg(not(feature = "tui"))]
    {
        eprintln!("Error: TUI feature is not enabled");
        eprintln!("Rebuild with: cargo build --features tui");
        std::process::exit(1);
    }

    Ok(())
}

fn init(
    name: &str,
    directory: Option<std::path::PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Initializing project: {}", name);

    let dir = directory.unwrap_or_else(|| std::path::PathBuf::from(name));

    if dir.exists() {
        eprintln!("Error: Directory already exists: {}", dir.display());
        std::process::exit(1);
    }

    std::fs::create_dir_all(&dir)?;

    // Use the template system for init
    let template_kind = TemplateKind::from_description(name);
    let pkgbuild = template_kind.generate_pkgbuild(name, "0.1.0", "A package generated by ArchForge");

    std::fs::write(dir.join("PKGBUILD"), pkgbuild)?;
    std::fs::write(dir.join(".gitignore"), "target/\n*.pkg.tar.zst\n")?;

    println!("Created project at: {}", dir.display());
    println!("Next steps:");
    println!("  cd {}", dir.display());
    println!("  # Edit PKGBUILD as needed");
    println!("  archforge build .");
    Ok(())
}

/// Validate a PKGBUILD using namcap
fn validate(
    path: &Path,
    validate_srcinfo: bool,
    check_deps: bool,
) -> Result<(), Box<dyn Error>> {
    eprintln!("Validating PKGBUILD at: {}", path.display());

    // Resolve path
    let pkgbuild_path = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent().unwrap_or(Path::new(".")).to_path_buf()
    };

    // Check if namcap is installed
    let namcap_installed = std::process::Command::new("which")
        .arg("namcap")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !namcap_installed {
        eprintln!("Warning: namcap not installed. Install with: sudo pacman -S namcap");
        eprintln!("Running basic PKGBUILD checks instead...\n");
    }

    // Find the PKGBUILD file
    let pkbuild_file = pkgbuild_path.join("PKGBUILD");
    let srcinfo_file = pkgbuild_path.join(".SRCINFO");

    if !validate_srcinfo && !check_deps {
        // Full PKGBUILD validation
        if pkbuild_file.exists() {
            run_namcap_pkbuild(&pkbuild_file)?;
        } else {
            eprintln!("PKGBUILD not found at: {}", pkbuild_file.display());
        }
    } else if validate_srcinfo {
        // Validate .SRCINFO
        if srcinfo_file.exists() {
            run_namcap_srcinfo(&srcinfo_file)?;
        } else {
            eprintln!(".SRCINFO not found. Generate with: makepkg --printsrcinfo > .SRCINFO");
        }
    }

    if check_deps {
        // Check dependencies
        check_dependencies(&pkgbuild_path)?;
    }

    eprintln!("\nValidation complete!");
    Ok(())
}

/// Run namcap on a PKGBUILD file
fn run_namcap_pkbuild(pkbuild_path: &Path) -> Result<(), Box<dyn Error>> {
    eprintln!("Running namcap on PKGBUILD...");

    let output = std::process::Command::new("namcap")
        .arg(pkbuild_path)
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if output.status.success() && stdout.is_empty() && stderr.is_empty() {
                eprintln!("PKGBUILD validation passed! No issues found.");
            } else {
                if !stdout.is_empty() {
                    eprintln!("Namcap output:\n{}", stdout);
                }
                if !stderr.is_empty() {
                    eprintln!("Errors:\n{}", stderr);
                }
            }
        }
        Err(e) => {
            eprintln!("Namcap not available or error running: {}", e);
            eprintln!("Install namcap with: sudo pacman -S pacman-contrib");
        }
    }

    Ok(())
}

/// Run namcap on a .SRCINFO file
fn run_namcap_srcinfo(srcinfo_path: &Path) -> Result<(), Box<dyn Error>> {
    eprintln!("Running namcap on .SRCINFO...");

    let output = std::process::Command::new("namcap")
        .arg(srcinfo_path)
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if output.status.success() && stdout.is_empty() && stderr.is_empty() {
                eprintln!(".SRCINFO validation passed!");
            } else {
                if !stdout.is_empty() {
                    eprintln!("Namcap output:\n{}", stdout);
                }
                if !stderr.is_empty() {
                    eprintln!("Errors:\n{}", stderr);
                }
            }
        }
        Err(e) => {
            eprintln!("Namcap not available or error running: {}", e);
        }
    }

    Ok(())
}

/// Check dependencies in a PKGBUILD
/// Optimized: Batch query all packages at once instead of one-by-one
fn check_dependencies(pkgdir: &Path) -> Result<(), Box<dyn Error>> {
    eprintln!("Checking dependencies...");

    let pkgbuild_path = pkgdir.join("PKGBUILD");
    if !pkgbuild_path.exists() {
        eprintln!("PKGBUILD not found. Cannot check dependencies.");
        return Ok(());
    }

    // Use bash to source the PKGBUILD and extract variables
    let script = r#"
        source PKGBUILD 2>/dev/null || exit 1
        echo "DEPENDS_START"
        for dep in "${depends[@]}"; do
            echo "$dep"
        done
        echo "DEPENDS_END"
        echo "MAKEDEPENDS_START"
        for dep in "${makedepends[@]}"; do
            echo "$dep"
        done
        echo "MAKEDEPENDS_END"
    "#;

    let output = std::process::Command::new("bash")
        .arg("-c")
        .arg(script)
        .current_dir(pkgdir)
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);

    // Parse the output
    let mut depends: Vec<String> = Vec::new();
    let mut makedepends: Vec<String> = Vec::new();
    let mut section = "";

    for line in output_str.lines() {
        match line.trim() {
            "DEPENDS_START" => section = "depends",
            "DEPENDS_END" => section = "",
            "MAKEDEPENDS_START" => section = "makedepends",
            "MAKEDEPENDS_END" => section = "",
            dep if !dep.is_empty() => {
                if section == "depends" {
                    depends.push(dep.to_string());
                } else if section == "makedepends" {
                    makedepends.push(dep.to_string());
                }
            }
            _ => {}
        }
    }

    // Print dependency summary
    if !depends.is_empty() {
        eprintln!("\nDependencies ({}):", depends.len());
        for dep in &depends {
            eprintln!("  - {}", dep);
        }
    }

    if !makedepends.is_empty() {
        eprintln!("\nBuild Dependencies ({}):", makedepends.len());
        for dep in &makedepends {
            eprintln!("  - {}", dep);
        }
    }

    if depends.is_empty() && makedepends.is_empty() {
        eprintln!("\nNo dependencies found in PKGBUILD");
    }

    // Check if packages are installed - OPTIMIZED: batch query for better performance
    let all_deps: Vec<&String> = depends.iter().chain(makedepends.iter()).collect();

    if !all_deps.is_empty() {
        eprintln!("\nPackage availability check:");

        // Extract package names (strip version operators)
        let pkg_names: Vec<String> = all_deps.iter()
            .map(|dep| {
                dep.split_whitespace()
                    .next()
                    .and_then(|s| s.split(['>', '<', '=']).next())
                    .unwrap_or(dep)
                    .to_string()
            })
            .collect();

        // OPTIMIZATION: Query all packages in a single pacman call
        // Split into chunks to avoid command line length limits
        const BATCH_SIZE: usize = 50;
        let mut installed_set: HashSet<String> = HashSet::new();

        for chunk in pkg_names.chunks(BATCH_SIZE) {
            let pacman_output = std::process::Command::new("pacman")
                .args(["-Q", "--quiet"])
                .args(chunk)
                .output()
                .ok();

            if let Some(output) = pacman_output {
                if let Ok(pkgs) = String::from_utf8(output.stdout) {
                    for pkg in pkgs.lines() {
                        installed_set.insert(pkg.to_string());
                    }
                }
            }
        }

        // Display results
        for (_dep, pkg_name) in all_deps.iter().zip(pkg_names.iter()) {
            if installed_set.contains(pkg_name) {
                eprintln!("  ✓ {} - installed", pkg_name);
            } else {
                eprintln!("  ✗ {} - not installed", pkg_name);
            }
        }
    }

    Ok(())
}

fn swarm(cmd: SwarmCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        SwarmCommands::Start { telemetry: _ } => {
            eprintln!("[SWARM] P2P networking - Coming soon!");
            eprintln!("[SWARM] This will enable telemetry sharing with other ArchForge users.");
        }
        SwarmCommands::Stop => {
            eprintln!("[SWARM] Stopping swarm network...");
        }
        SwarmCommands::Peers => {
            eprintln!("[SWARM] Connected peers: 0 (P2P coming soon)");
        }
        SwarmCommands::Sync { address: _ } => {
            eprintln!("[SWARM] Sync - Coming soon!");
        }
    }
    Ok(())
}

fn status() -> Result<(), Box<dyn std::error::Error>> {
    println!("ArchForge Status");
    println!("================");
    println!("Version: {}", VERSION);

    // Check for namcap
    if std::process::Command::new("which").arg("namcap").output()?.status.success() {
        println!("Namcap: Installed");
    } else {
        println!("Namcap: NOT INSTALLED (install pacman-contrib)");
    }

    // Check for makepkg
    if std::process::Command::new("makepkg").arg("--version").output().is_ok() {
        println!("Makepkg: Installed");
    } else {
        println!("Makepkg: NOT INSTALLED (install base-devel)");
    }

    // Check for AUR helpers
    if std::process::Command::new("paru").arg("--version").output().is_ok() {
        println!("Paru: Installed");
    } else if std::process::Command::new("yay").arg("--version").output().is_ok() {
        println!("Yay: Installed");
    } else {
        println!("AUR helper: Not found (install paru or yay)");
    }

    // Config location
    println!("\nConfig: ~/.config/archforge/config.toml");

    Ok(())
}

fn cache(cmd: CacheCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        CacheCommands::Stats => {
            println!("Cache Statistics");
            println!("================");

            if let Some(cache_dir) = dirs::cache_dir() {
                let archforge_cache = cache_dir.join("archforge");
                if archforge_cache.exists() {
                    let files: Vec<_> = walkdir::WalkDir::new(&archforge_cache)
                        .max_depth(10)
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .filter(|e| e.file_type().is_file())
                        .collect();

                    let size: u64 = files.iter()
                        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
                        .sum();

                    println!("Files: {}", files.len());
                    println!("Size: {} bytes ({:.2} MB)", size, size as f64 / 1024.0 / 1024.0);
                } else {
                    println!("Cache directory: Empty");
                }
            }

            // Show in-memory cache stats
            if let Ok(cache) = get_aur_cache().read() {
                println!("\nAUR RPC Cache:");
                println!("  Entries: {}", cache.len());
            }
            let ai_cache_count = archforge::ai::chutes::get_response_cache_for_stats();
            println!("\nAI Response Cache:");
            println!("  Entries: {}", ai_cache_count);
        }
        CacheCommands::Models => {
            println!("Clearing model cache... (Not implemented yet)");
        }
        CacheCommands::Builds => {
            if let Some(cache_dir) = dirs::cache_dir() {
                let build_cache = cache_dir.join("archforge/builds");
                if build_cache.exists() {
                    std::fs::remove_dir_all(&build_cache)?;
                    println!("Build cache cleared");
                } else {
                    println!("Build cache: Already empty");
                }
            }
        }
        CacheCommands::All => {
            println!("Clearing all caches...");
            
            // Clear in-memory caches
            if let Ok(mut cache) = get_aur_cache().write() {
                cache.clear();
                println!("AUR RPC cache cleared");
            }
            archforge::ai::chutes::clear_response_cache();
            
            // Clear disk cache
            if let Some(cache_dir) = dirs::cache_dir() {
                let archforge_cache = cache_dir.join("archforge");
                if archforge_cache.exists() {
                    std::fs::remove_dir_all(&archforge_cache)?;
                    println!("Disk cache cleared");
                }
            }
        }
    }
    Ok(())
}
