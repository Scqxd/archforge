//! Build engine - makepkg/yay/paru wrapper with real-time output
//!
//! Features:
//! - Async makepkg execution with progress
//! - Dependency checking and resolution
//! - AUR helper integration (yay/paru)
//! - Build queue management
//! - Output streaming

use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use tokio::process::Command as AsyncCommand;
use tokio::sync::mpsc;
use tokio::io::{AsyncBufReadExt, BufReader};
use futures::stream::{self, StreamExt};
use thiserror::Error;
use tracing::{info, debug, error, warn};
use indicatif::{ProgressBar, ProgressStyle};

pub use makepkg::Makepkg;
pub use aur::{AURHelper, AURHelperType};

mod makepkg;
mod aur;

/// Build configuration
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Path to makepkg binary
    pub makepkg_path: PathBuf,

    /// AUR helper type
    pub aur_helper: AURHelperType,

    /// Build directory
    pub build_dir: PathBuf,

    /// Number of parallel jobs (makepkg -j)
    pub jobs: u32,

    /// Skip dependency checks
    pub skip_deps: bool,

    /// Skip PGP signature checks
    pub skip_pgp: bool,

    /// Install after building
    pub install: bool,

    /// Keep build files (no rm -rf)
    pub keep: bool,

    /// Run as fakeroot
    pub fakeroot: bool,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            makepkg_path: PathBuf::from("/usr/bin/makepkg"),
            aur_helper: AURHelperType::Paru,
            build_dir: std::env::temp_dir().join("archforge-builds"),
            jobs: num_cpus::get() as u32,
            skip_deps: false,
            skip_pgp: false,
            install: false,
            keep: false,
            fakeroot: false,
        }
    }
}

/// Build result
#[derive(Debug, Clone)]
pub struct BuildResult {
    /// Package name
    pub package_name: String,

    /// Path to built package(s)
    pub package_paths: Vec<PathBuf>,

    /// Build logs
    pub logs: String,

    /// Build time in seconds
    pub build_time: u64,

    /// Whether build was successful
    pub success: bool,

    /// Exit status
    pub exit_status: ExitStatus,
}

/// Build error types
#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Makepkg not found at {0}")]
    MakepkgNotFound(PathBuf),

    #[error("AUR helper not found: {0}")]
    AURHelperNotFound(String),

    #[error("Build failed with exit code {code}")]
    BuildFailed { code: Option<i32> },

    #[error("Dependency error: {0}")]
    DependencyError(String),

    #[error("Package not found: {0}")]
    PackageNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Cancelled by user")]
    Cancelled,

    #[error("IO error: {source}")]
    Io { source: std::io::Error },
}

impl From<std::io::Error> for BuildError {
    fn from(e: std::io::Error) -> Self {
        Self::Io { source: e }
    }
}

/// AUR package info
#[derive(Debug, Clone)]
pub struct AURPackage {
    pub name: String,
    pub version: String,
    pub description: String,
    pub maintainer: String,
    pub votes: u32,
    pub popularity: f64,
    pub depends: Vec<String>,
    pub makedepends: Vec<String>,
    pub optdepends: Vec<String>,
    pub url: String,
}

/// Build queue item
#[derive(Debug, Clone)]
pub struct BuildQueueItem {
    /// Package path or name
    pub path: PathBuf,

    /// Status
    pub status: BuildStatus,

    /// Position in queue
    pub position: usize,

    /// Result if completed
    pub result: Option<BuildResult>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuildStatus {
    Pending,
    Downloading,
    Building,
    Installing,
    Completed,
    Failed,
}

/// Progress update from builder
#[derive(Debug, Clone)]
pub enum BuildProgress {
    Started(String),
    Downloading(String),
    Extracting(String),
    Building(String),
    Installing(String),
    Completed(String),
    Failed(String, String),
    Progress(String, u64, u64),
}

/// Stream builder output
pub async fn stream_build_output(
    mut cmd: tokio::process::Command,
    tx: mpsc::Sender<BuildProgress>,
) -> Result<String, BuildError> {
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| BuildError::Io { source: e })?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let mut out = String::new();

    // Read stdout and stderr concurrently
    let (tx1, mut rx1) = mpsc::channel::<String>(100);
    let (tx2, mut rx2) = mpsc::channel::<String>(100);
    let tx1 = std::sync::Arc::new(tx1);
    let tx2 = std::sync::Arc::new(tx2);

    // Spawn reading tasks
    let stdout_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.line().await {
            let _ = tx1.send(format!("[OUT] {}", line)).await;
        }
    });

    let stderr_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.line().await {
            let _ = tx2.send(format!("[ERR] {}", line)).await;
        }
    });

    // Merge outputs
    loop {
        tokio::select! {
            Some(line) = rx1.recv() => {
                out.push_str(&line);
                out.push('\n');
                if line.contains("==> Making package:") {
                    let _ = tx.send(BuildProgress::Started(line.clone())).await;
                } else if line.contains("==> Retrieving sources") {
                    let _ = tx.send(BuildProgress::Downloading("Fetching sources".to_string())).await;
                } else if line.contains("==> Extracting sources") {
                    let _ = tx.send(BuildProgress::Extracting("Extracting".to_string())).await;
                } else if line.contains("==> Starting build()") {
                    let _ = tx.send(BuildProgress::Building("Building".to_string())).await;
                } else if line.contains("==> Entering fakeroot environment") {
                    let _ = tx.send(BuildProgress::Building("Building (fakeroot)".to_string())).await;
                } else if line.contains("==> Installing package") {
                    let _ = tx.send(BuildProgress::Installing("Installing".to_string())).await;
                } else if line.contains("==> Finished making") {
                    let _ = tx.send(BuildProgress::Completed(line.clone())).await;
                }
            }
            Some(line) = rx2.recv() => {
                out.push_str(&line);
                out.push('\n');
                if line.contains("error") {
                    let _ = tx.send(BuildProgress::Failed("Build error".to_string(), line)).await;
                }
            }
            result = child.wait() => {
                stdout_task.abort();
                stderr_task.abort();
                break result;
            }
        }
    };

    let status = match child.wait().await {
        Ok(status) => status,
        Err(e) => return Err(BuildError::Io { source: e }),
    };

    if status.success() {
        Ok(out)
    } else {
        Err(BuildError::BuildFailed { code: status.code() })
    }
}

/// Check for missing dependencies
pub async fn check_missing_deps(pkgbuild_dir: &Path) -> Vec<String> {
    let mut missing = Vec::new();

    // Run makepkg --nodeps to see what's missing
    let output = Command::new("makepkg")
        .args(&["--nobuild", "--nodeps"])
        .current_dir(pkgbuild_dir)
        .output()
        .await
        .ok();

    if let Some(output) = output {
        let stderr = String::from_utf8_lossy(&output.stderr);
        for line in stderr.lines() {
            if line.contains("could not be found") || line.contains("missing") {
                // Extract package name
                if let Some(start) = line.find('\'') {
                    if let Some(end) = line[start+1..].find('\'') {
                        let pkg = &line[start+1..start+1+end];
                        if !pkg.is_empty() && !missing.contains(&pkg.to_string()) {
                            missing.push(pkg.to_string());
                        }
                    }
                }
            }
        }
    }

    missing
}

/// Resolve dependencies via pacman
pub async fn resolve_deps(deps: &[String]) -> Vec<String> {
    let mut resolved = Vec::new();
    let mut unresolved = Vec::new();

    for dep in deps {
        // Check if installed
        let output = Command::new("pacman")
            .args(&["-Qq", dep])
            .output()
            .await
            .ok();

        if output.map(|o| o.status.success()).unwrap_or(false) {
            resolved.push(format!("{} (installed)", dep));
        } else {
            // Try to find in repos
            let output = Command::new("pacman")
                .args(&["-Ss", dep])
                .output()
                .await
                .ok();

            if output.map(|o| o.status.success()).unwrap_or(false) {
                resolved.push(format!("{} (in repos)", dep));
            } else {
                unresolved.push(dep.clone());
            }
        }
    }

    // Print resolution
    for r in &resolved {
        debug!("Dependency resolved: {}", r);
    }
    for u in &unresolved {
        warn!("Unresolved dependency: {}", u);
    }

    unresolved
}

/// Install dependencies
pub async fn install_deps(deps: &[String]) -> Result<(), BuildError> {
    if deps.is_empty() {
        return Ok(());
    }

    info!("Installing dependencies: {:?}", deps);

    // Use pacman for official deps, AUR helper for AUR deps
    let status = Command::new("sudo")
        .args(&["pacman", "-S", "--noconfirm"])
        .args(deps)
        .status()
        .await
        .map_err(|e| BuildError::Io { source: e })?;

    if status.success() {
        Ok(())
    } else {
        Err(BuildError::DependencyError(format!(
            "Failed to install: {}",
            deps.join(", ")
        )))
    }
}