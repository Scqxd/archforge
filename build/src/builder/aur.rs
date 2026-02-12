//! AUR helper integration (paru/yay)
//!
//! Provides search, info, and build operations via AUR helpers

use super::{AURPackage, AURHelperType, BuildError};
use std::process::Command;
use tracing::debug;
use serde::Deserialize;

#[derive(Debug)]
pub struct AURHelper {
    helper_type: AURHelperType,
    command: String,
}

#[derive(Deserialize, Debug)]
struct AURJsonResult {
    Name: String,
    Version: String,
    Description: String,
    Maintainer: String,
    Votes: u32,
    Popularity: f64,
    Depends: Option<Vec<String>>,
    MakeDepends: Option<Vec<String>>,
    OptDepends: Option<Vec<String>>,
    URL: String,
}

#[derive(Deserialize, Debug)]
struct AURJsonResponse {
    results: Vec<AURJsonResult>,
    type_: String,
    version: u32,
}

impl AURHelper {
    /// Create new AUR helper
    pub fn new(helper_type: AURHelperType) -> Self {
        let command = match helper_type {
            AURHelperType::Paru => "paru",
            AURHelperType::Yay => "yay",
            AURHelperType::Custom(path) => path.to_str().unwrap_or("custom"),
        };

        Self { helper_type, command: command.to_string() }
    }

    /// Check if helper is available
    pub fn is_available(&self) -> bool {
        which::which(&self.command).is_ok()
    }

    /// Search AUR for packages
    pub async fn search(&self, query: &str) -> Result<Vec<AURPackage>, BuildError> {
        debug!("Searching AUR for: {}", query);

        // First try the AUR RPC directly (faster)
        if let Ok(packages) = self.search_aur_rpc(query).await {
            return Ok(packages);
        }

        // Fallback to helper
        self.search_with_helper(query).await
    }

    /// Search using AUR RPC directly
    async fn search_aur_rpc(&self, query: &str) -> Result<Vec<AURPackage>, BuildError> {
        let url = format!(
            "https://aur.archlinux.org/rpc.php?v=5&type=search&arg={}",
            query.replace(' ', "%20")
        );

        let response = reqwest::blocking::get(&url)
            .map_err(|e| BuildError::BuildFailed { code: None })?
            .json::<AURJsonResponse>()
            .map_err(|e| BuildError::BuildFailed { code: None })?;

        Ok(response.results.into_iter().map(|p| p.into()).collect())
    }

    /// Search using helper (paru/yay)
    async fn search_with_helper(&self, query: &str) -> Result<Vec<AURPackage>, BuildError> {
        let output = Command::new(&self.command)
            .args(&["-Ss", "--json", query])
            .output()
            .map_err(|e| BuildError::BuildFailed { code: None })?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let packages: Vec<AURPackage> = serde_json::from_slice(&output.stdout)
            .map_err(|e| BuildError::BuildFailed { code: None })?;

        Ok(packages)
    }

    /// Get info for a specific package
    pub async fn info(&self, package: &str) -> Result<Option<AURPackage>, BuildError> {
        debug!("Getting AUR info for: {}", package);

        // Try AUR RPC
        let url = format!(
            "https://aur.archlinux.org/rpc.php?v=5&type=info&arg={}",
            urlencoding::encode(package)
        );

        let response = reqwest::blocking::get(&url)
            .map_err(|e| BuildError::BuildFailed { code: None })?
            .json::<AURJsonResponse>()
            .map_err(|e| BuildError::BuildFailed { code: None })?;

        Ok(response.results.into_iter().next().map(|p| p.into()))
    }

    /// Build and install from AUR
    pub async fn build_and_install(&self, package: &str) -> Result<(), BuildError> {
        info!("Building and installing from AUR: {}", package);

        let output = Command::new(&self.command)
            .args(&["-S", "--noconfirm", "--needed", "--noedit", package])
            .output()
            .map_err(|e| BuildError::BuildFailed { code: None })?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(BuildError::BuildFailed { code: output.status.code().map(|c| c as i32) })
        }
    }

    /// Get system package version
    pub async fn get_version(&self, package: &str) -> Result<Option<String>, BuildError> {
        let output = Command::new("pacman")
            .args(&["-Qi", package])
            .output()
            .map_err(|e| BuildError::BuildFailed { code: None })?;

        if !output.status.success() {
            return Ok(None);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.starts_with("Version") {
                if let Some(ver) = line.split(':').nth(1) {
                    return Ok(Some(ver.trim().to_string()));
                }
            }
        }

        Ok(None)
    }

    /// Download AUR package sources
    pub async fn download_sources(&self, package: &str) -> Result<(), BuildError> {
        info!("Downloading sources for: {}", package);

        let output = Command::new(&self.command)
            .args(&["-G", package])
            .current_dir(std::env::temp_dir())
            .output()
            .map_err(|e| BuildError::BuildFailed { code: None })?;

        if output.status.success() {
            Ok(())
        } else {
            Err(BuildError::BuildFailed { code: output.status.code().map(|c| c as i32) })
        }
    }
}

impl From<AURJsonResult> for AURPackage {
    fn from(aur: AURJsonResult) -> Self {
        Self {
            name: aur.Name,
            version: aur.Version,
            description: aur.Description,
            maintainer: aur.Maintainer,
            votes: aur.Votes,
            popularity: aur.Popularity,
            depends: aur.Depends.unwrap_or_default(),
            makedepends: aur.MakeDepends.unwrap_or_default(),
            optdepends: aur.OptDepends.unwrap_or_default(),
            url: aur.URL,
        }
    }
}

impl Default for AURHelper {
    fn default() -> Self {
        Self::new(AURHelperType::Paru)
    }
}