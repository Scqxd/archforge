//! Makepkg wrapper for building packages

use super::{BuildConfig, BuildResult, BuildError, BuildProgress};
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::{debug, info, error};
use mpsc::Sender;

#[derive(Debug)]
pub struct Makepkg {
    config: BuildConfig,
}

impl Makepkg {
    /// Create new makepkg wrapper
    pub fn new(config: BuildConfig) -> Self {
        Self { config }
    }

    /// Check if makepkg is available
    pub fn is_available(&self) -> bool {
        self.config.makepkg_path.exists() && self.config.makepkg_path.is_file()
    }

    /// Validate a PKGBUILD without building
    pub async fn validate(&self, pkgbuild_dir: &Path) -> Result<bool, BuildError> {
        info!("Validating PKGBUILD in: {:?}", pkgbuild_dir);

        let output = Command::new(&self.config.makepkg_path)
            .args(&["--nobuild", "--nodeps"])
            .current_dir(pkgbuild_dir)
            .output()
            .await
            .map_err(|e| BuildError::Io { source: e })?;

        if output.status.success() {
            Ok(true)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("PKGBUILD validation failed: {}", stderr);
            Ok(false)
        }
    }

    /// Build a package
    pub async fn build(&self, pkgbuild_dir: &Path, tx: Option<&Sender<BuildProgress>>) -> Result<BuildResult, BuildError> {
        if !self.is_available() {
            return Err(BuildError::MakepkgNotFound(self.config.makepkg_path.clone()));
        }

        let start_time = std::time::Instant::now();
        info!("Building package in: {:?}", pkgbuild_dir);

        // Build command
        let mut args = vec!["--noconfirm", "--skippgpcheck"];

        if self.config.skip_deps {
            args.push("--nodeps");
        }

        if self.config.jobs > 1 {
            args.push("--jobs");
            args.push(&self.config.jobs.to_string());
        }

        if self.config.keep {
            args.push("--keep");
        }

        if self.config.fakeroot {
            args.push("--fakeroot");
        }

        // Add install flag
        if self.config.install {
            args.push("--install");
        }

        // Prepare command
        let mut cmd = Command::new(&self.config.makepkg_path);
        cmd.args(&args)
            .current_dir(pkgbuild_dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        debug!("Running: {:?} {:?}", &self.config.makepkg_path, args);

        // Stream output
        let output = if let Some(tx) = tx {
            super::stream_build_output(cmd, tx.clone()).await?
        } else {
            let output = cmd.output().await.map_err(|e| BuildError::Io { source: e })?;
            String::from_utf8_lossy(&output.stdout).to_string()
        };

        let build_time = start_time.elapsed().as_secs();

        // Find built packages
        let package_paths = self.find_packages(pkgbuild_dir)?;

        // Get package name from PKGBUILD
        let pkgname = self.get_pkgname(pkgbuild_dir)?;

        Ok(BuildResult {
            package_name: pkgname,
            package_paths,
            logs: output,
            build_time,
            success: true,
            exit_status: std::process::ExitStatus::default(),
        })
    }

    /// Find built .pkg.tar.zst files
    fn find_packages(&self, dir: &Path) -> Result<Vec<PathBuf>, BuildError> {
        let mut packages = Vec::new();

        let entries = std::fs::read_dir(dir)
            .map_err(|e| BuildError::Io { source: e })?;

        for entry in entries {
            let entry = entry.map_err(|e| BuildError::Io { source: e })?;
            let path = entry.path();

            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy();
                if ext_str.ends_with("pkg.tar.zst") ||
                   ext_str.ends_with("pkg.tar.xz") ||
                   ext_str.ends_with("pkg.tar.gz") {
                    packages.push(path);
                }
            }
        }

        Ok(packages)
    }

    /// Extract pkgname from PKGBUILD
    fn get_pkgname(&self, dir: &Path) -> Result<String, BuildError> {
        let pkgbuild_path = dir.join("PKGBUILD");
        let content = std::fs::read_to_string(&pkgbuild_path)
            .map_err(|e| BuildError::Io { source: e })?;

        for line in content.lines() {
            if let Some(name) = line.strip_prefix("pkgname=") {
                return Ok(name.trim().to_string());
            }
        }

        Err(BuildError::PackageNotFound("pkgname not found".to_string()))
    }

    /// Get missing dependencies
    pub async fn get_missing_deps(&self, pkgbuild_dir: &Path) -> Vec<String> {
        super::check_missing_deps(pkgbuild_dir).await
    }

    /// Clean build directory
    pub async fn clean(&self, pkgbuild_dir: &Path) -> Result<(), BuildError> {
        let output = Command::new(&self.config.makepkg_path)
            .args(&["--clean", "--noconfirm"])
            .current_dir(pkgbuild_dir)
            .output()
            .await
            .map_err(|e| BuildError::Io { source: e })?;

        if output.status.success() {
            Ok(())
        } else {
            Err(BuildError::BuildFailed { code: output.status.code().map(|c| c as i32) })
        }
    }
}