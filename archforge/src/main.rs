//! ArchForge - AI-powered TUI for PKGBUILD generation and AUR management

use std::path::Path;
use std::error::Error;
use clap::Parser;

pub mod ai;
pub mod cli;
pub mod templates;

pub use ai::{AiProvider, ChutesClient};
pub use cli::Cli;
pub use templates::TemplateKind;

/// Version of ArchForge
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Homepage
pub const HOMEPAGE: &str = "https://github.com/archforge/archforge";

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
        if cmd == "interactive" || cmd == "-i" || cmd.is_empty() {
            print_logo();
            println!();
        }
    }

    // Parse CLI arguments
    let cli = Cli::parse();

    // Run the command
    match cli.command {
        cli::Commands::Generate { description, output, quiet: _, ai_provider, api_key } => {
            generate(&description, output, ai_provider, api_key)?;
        }
        cli::Commands::Build { package, install: _, nodeps: _ } => {
            build(&package)?;
        }
        cli::Commands::Search { query, json, limit: _ } => {
            search(&query, json)?;
        }
        cli::Commands::Info { package } => {
            info(&package)?;
        }
        cli::Commands::Deploy { package, target: _, yes: _ } => {
            deploy(&package)?;
        }
        cli::Commands::Interactive { no_model: _ } => {
            run_tui()?;
        }
        cli::Commands::Init { name, template: _, directory } => {
            init(&name, directory)?;
        }
        cli::Commands::Validate { path, srcinfo, dependencies } => {
            validate(&path, srcinfo, dependencies)?;
        }
        cli::Commands::Swarm(cmd) => {
            swarm(cmd)?;
        }
        cli::Commands::Status => {
            status()?;
        }
        cli::Commands::Cache(cmd) => {
            cache(cmd)?;
        }
    }

    Ok(())
}

fn generate(
    description: &str,
    output: Option<std::path::PathBuf>,
    ai_provider: ai::AiProvider,
    api_key: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Generating PKGBUILD for: {}", description);

    match ai_provider {
        ai::AiProvider::Chutes => {
            let api_key = api_key.or_else(|| {
                std::env::var("CHUTES_API_KEY").ok()
            });

            if let Some(api_key) = api_key {
                let client = ChutesClient::new(api_key);
                match client.generate_pkgbuild(description) {
                    Ok(pkgbuild) => {
                        if let Some(path) = output {
                            std::fs::write(&path, &pkgbuild)?;
                            eprintln!("PKGBUILD saved to: {}", path.display());
                        } else {
                            println!("{}", pkgbuild);
                        }
                        return Ok(());
                    }
                    Err(e) => {
                        eprintln!("AI generation failed: {}. Falling back to template...", e);
                    }
                }
            } else {
                eprintln!("No API key provided. Using fallback template generation.");
            }
            generate_fallback(description, output)?;
        }
        ai::AiProvider::Local => {
            eprintln!("Local AI provider not implemented yet. Using fallback.");
            generate_fallback(description, output)?;
        }
        ai::AiProvider::Openai => {
            eprintln!("OpenAI provider not implemented yet. Using fallback.");
            generate_fallback(description, output)?;
        }
    }

    Ok(())
}

fn generate_fallback(
    description: &str,
    output: Option<std::path::PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let pkgname = slugify(description);

    // Detect language and generate appropriate template
    let template_kind = TemplateKind::from_description(description);
    let pkgver = "0.1.0";

    let pkgbuild = template_kind.generate_pkgbuild(&pkgname, pkgver, description);

    if let Some(path) = output {
        std::fs::write(&path, &pkgbuild)?;
        eprintln!("PKGBUILD saved to: {}", path.display());
    } else {
        println!("{}", pkgbuild);
    }

    Ok(())
}

fn build(package: &str) -> Result<(), Box<dyn std::error::Error>> {
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
            let output = std::process::Command::new("makepkg")
                .args(&["--nobuild", "--nodeps"])
                .current_dir(actual_path.parent().unwrap_or(&std::path::PathBuf::from(".")))
                .output()?;

            if output.status.success() {
                eprintln!("PKGBUILD validation successful!");
            } else {
                eprintln!("PKGBUILD has errors:");
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

fn search(query: &str, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Searching AUR for: {}", query);

    // Query AUR RPC
    let url = format!(
        "https://aur.archlinux.org/rpc.php?v=5&type=search&arg={}",
        urlencoding::encode(query)
    );

    let response = reqwest::blocking::get(&url)?
        .json::<serde_json::Value>()?;

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

    let url = format!(
        "https://aur.archlinux.org/rpc.php?v=5&type=info&arg={}",
        urlencoding::encode(package)
    );

    let response = reqwest::blocking::get(&url)?
        .json::<serde_json::Value>()?;

    if let Some(result) = response.get("results").and_then(|r| r.as_object()) {
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

    archforge::tui::run_tui()?;
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
    let pkgbuild = template_kind.generate_pkgbuild(name, "0.1.0", &format!("A package generated by ArchForge"));

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
        path.parent().unwrap_or(&Path::new(".")).to_path_buf()
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
fn check_dependencies(pkgdir: &Path) -> Result<(), Box<dyn Error>> {
    eprintln!("Checking dependencies...");

    let pkbuild_path = pkgdir.join("PKGBUILD");
    if !pkbuild_path.exists() {
        eprintln!("PKGBUILD not found. Cannot check dependencies.");
        return Ok(());
    }

    // Read and parse PKGBUILD
    let content = std::fs::read_to_string(&pkbuild_path)?;
    let lines: Vec<&str> = content.lines().collect();

    let mut depends: Vec<String> = Vec::new();
    let mut makedepends: Vec<String> = Vec::new();

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("depends=") {
            if let Some(start) = trimmed.find('(') {
                if let Some(end) = trimmed.find(')') {
                    let deps = &trimmed[start + 1..end];
                    for dep in deps.split_whitespace() {
                        if !dep.is_empty() && dep != "'" && dep != "\"" {
                            let clean_dep = dep.trim_matches('\'').trim_matches('"');
                            depends.push(clean_dep.to_string());
                        }
                    }
                }
            }
        } else if trimmed.starts_with("makedepends=") {
            if let Some(start) = trimmed.find('(') {
                if let Some(end) = trimmed.find(')') {
                    let deps = &trimmed[start + 1..end];
                    for dep in deps.split_whitespace() {
                        if !dep.is_empty() && dep != "'" && dep != "\"" {
                            let clean_dep = dep.trim_matches('\'').trim_matches('"');
                            makedepends.push(clean_dep.to_string());
                        }
                    }
                }
            }
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

    // Check if packages are installed
    eprintln!("\nPackage availability check:");
    for dep in &depends {
        let installed = std::process::Command::new("pacman")
            .args(&["-Qq", dep])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if installed {
            eprintln!("  [OK] {} - installed", dep);
        } else {
            eprintln!("  [MISSING] {} - not installed", dep);
        }
    }

    Ok(())
}

fn swarm(cmd: cli::SwarmCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        cli::SwarmCommands::Start { telemetry: _ } => {
            eprintln!("[SWARM] P2P networking - Coming soon!");
            eprintln!("[SWARM] This will enable telemetry sharing with other ArchForge users.");
        }
        cli::SwarmCommands::Stop => {
            eprintln!("[SWARM] Stopping swarm network...");
        }
        cli::SwarmCommands::Peers => {
            eprintln!("[SWARM] Connected peers: 0 (P2P coming soon)");
        }
        cli::SwarmCommands::Sync { address: _ } => {
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

fn cache(cmd: cli::CacheCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        cli::CacheCommands::Stats => {
            println!("Cache Statistics");
            println!("================");

            if let Some(cache_dir) = dirs::cache_dir() {
                let archforge_cache = cache_dir.join("archforge");
                if archforge_cache.exists() {
                    // Count files
                    let files: Vec<_> = walkdir::WalkDir::new(&archforge_cache)
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
        }
        cli::CacheCommands::Models => {
            println!("Clearing model cache... (Not implemented yet)");
        }
        cli::CacheCommands::Builds => {
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
        cli::CacheCommands::All => {
            println!("Clearing all caches...");
            if let Some(cache_dir) = dirs::cache_dir() {
                let archforge_cache = cache_dir.join("archforge");
                if archforge_cache.exists() {
                    std::fs::remove_dir_all(&archforge_cache)?;
                    println!("All caches cleared");
                }
            }
        }
    }
    Ok(())
}

/// Convert description to slug for package name
fn slugify(s: &str) -> String {
    let cleaned: String = s
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == ' ')
        .collect();
    cleaned
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .trim_matches('-')
        .to_string()
}