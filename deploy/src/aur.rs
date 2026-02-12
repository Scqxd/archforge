//! AUR deployment agent
//!
//! Provides:
//! - AUR package upload via git
//! - SSH key management
//! - PKGBUILD validation before upload
//! - Automatic .SRCINFO generation

use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;
use tracing::{info, debug, error, warn};

#[derive(Debug, Error)]
pub enum AURDeployError {
    #[error("Not a git repository: {0}")]
    NotGitRepo(PathBuf),

    #[error("SSH key not found: {0}")]
    SSKKeyNotFound(String),

    #[error("Git push failed: {0}")]
    GitPushFailed(String),

    #[error("AUR RPC error: {0}")]
    RPCError(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Package validation failed: {0}")]
    ValidationFailed(String),

    #[error("IO error: {0}")]
    IOError(String),
}

impl From<std::io::Error> for AURDeployError {
    fn from(e: std::io::Error) -> Self {
        AURDeployError::IOError(e.to_string())
    }
}

/// AUR uploader
pub struct AURUploader {
    /// SSH key path
    ssh_key: Option<PathBuf>,

    /// AUR username
    username: String,

    /// Commit message template
    commit_template: String,
}

impl AURUploader {
    /// Create new uploader
    pub fn new(username: String, ssh_key: Option<PathBuf>) -> Self {
        Self {
            username,
            ssh_key,
            commit_template: "Update {pkgname} to {pkgver}".to_string(),
        }
    }

    /// Deploy package to AUR
    pub async fn deploy(&self, pkgdir: &Path) -> Result<(), AURDeployError> {
        info!("Deploying {} to AUR", pkgdir.display());

        // Validate package directory
        if !pkgdir.exists() {
            return Err(AURDeployError::ValidationFailed(
                format!("Directory not found: {}", pkgdir.display())
            ));
        }

        let pkgbuild_path = pkgdir.join("PKGBUILD");
        if !pkgbuild_path.exists() {
            return Err(AURDeployError::ValidationFailed(
                "PKGBUILD not found".to_string()
            ));
        }

        // Get package name and version
        let (pkgname, pkgver) = self.parse_pkgbuild_info(pkgdir)?;

        info!("Package: {} v{}", pkgname, pkgver);

        // Check if it's a git repository
        let repo_path = pkgdir.join(".git");
        let is_git = repo_path.exists();

        if is_git {
            self.push_updates(pkgdir, &pkgname, &pkgver).await?;
        } else {
            warn!("Not a git repository, creating one...");
            self.create_new_aur_repo(pkgdir, &pkgname).await?;
        }

        Ok(())
    }

    /// Parse PKGBUILD for name and version
    fn parse_pkgbuild_info(&self, pkgdir: &Path) -> Result<(String, String), AURDeployError> {
        let pkgbuild_path = pkgdir.join("PKGBUILD");
        let content = std::fs::read_to_string(&pkgbuild_path)
            .map_err(|e| AURDeployError::ValidationFailed(e.to_string()))?;

        let mut pkgname = None;
        let mut pkgver = None;

        for line in content.lines() {
            if let Some(name) = line.strip_prefix("pkgname=") {
                pkgname = Some(name.trim().to_string());
            }
            if let Some(ver) = line.strip_prefix("pkgver=") {
                pkgver = Some(ver.trim().to_string());
            }
        }

        match (pkgname, pkgver) {
            (Some(n), Some(v)) => Ok((n, v)),
            _ => Err(AURDeployError::ValidationFailed(
                "pkgname or pkgver not found in PKGBUILD".to_string()
            )),
        }
    }

    /// Push updates to existing AUR repo
    async fn push_updates(&self, pkgdir: &Path, pkgname: &str, pkgver: &str) -> Result<(), AURDeployError> {
        let commit_msg = self.commit_template
            .replace("{pkgname}", pkgname)
            .replace("{pkgver}", pkgver);

        // Stage changes
        let output = Command::new("git")
            .args(&["add", "."])
            .current_dir(pkgdir)
            .output()
            .map_err(|e| AURDeployError::GitPushFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(AURDeployError::GitPushFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        // Commit
        let output = Command::new("git")
            .args(&["commit", "-m", &commit_msg])
            .current_dir(pkgdir)
            .output()
            .map_err(|e| AURDeployError::GitPushFailed(e.to_string()))?;

        if !output.status.success() {
            // Check if there are actual changes
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("nothing to commit") {
                info!("No changes to commit for {}", pkgname);
                return Ok(());
            }
            return Err(AURDeployError::GitPushFailed(stderr.to_string()));
        }

        // Push to AUR
        let remote = format!("ssh://aur@aur.archlinux.org/{}.git", pkgname);

        let mut cmd = Command::new("git");
        cmd.args(&["push", "aur", "master"])
            .current_dir(pkgdir);

        // Use SSH key if provided
        if let Some(ref key) = self.ssh_key {
            cmd.env("GIT_SSH_COMMAND", format!("ssh -i {}", key.display()));
        }

        let output = cmd.output()
            .map_err(|e| AURDeployError::GitPushFailed(e.to_string()))?;

        if output.status.success() {
            info!("Successfully pushed {} to AUR", pkgname);
            Ok(())
        } else {
            Err(AURDeployError::GitPushFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ))
        }
    }

    /// Create new AUR repository
    async fn create_new_aur_repo(&self, pkgdir: &Path, pkgname: &str) -> Result<(), AURDeployError> {
        info!("Creating new AUR repository for {}", pkgname);

        // Initialize git repo
        let output = Command::new("git")
            .args(&["init"])
            .current_dir(pkgdir)
            .output()
            .map_err(|e| AURDeployError::GitPushFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(AURDeployError::GitPushFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        // Add remote
        let remote = format!("ssh://aur@aur.archlinux.org/{}.git", pkgname);
        let output = Command::new("git")
            .args(&["remote", "add", "origin", &remote])
            .current_dir(pkgdir)
            .output()
            .map_err(|e| AURDeployError::GitPushFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(AURDeployError::GitPushFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        // Initial commit
        let output = Command::new("git")
            .args(&["add", "."])
            .current_dir(pkgdir)
            .output()
            .map_err(|e| AURDeployError::GitPushFailed(e.to_string()))?;

        let commit_msg = format!("Initial commit of {}", pkgname);
        let output = Command::new("git")
            .args(&["commit", "-m", &commit_msg])
            .current_dir(pkgdir)
            .output()
            .map_err(|e| AURDeployError::GitPushFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(AURDeployError::GitPushFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        // Push (this will fail if repo doesn't exist on AUR)
        info!("Run the following to push to AUR:");
        info!("  cd {}", pkgdir.display());
        info!("  git push -u origin master");
        info!("\nMake sure you have SSH access to AUR configured.");

        Ok(())
    }

    /// Generate .SRCINFO
    pub fn generate_srcinfo(&self, pkgdir: &Path) -> Result<(), AURDeployError> {
        let output = Command::new("makepkg")
            .args(&["--printsrcinfo"])
            .current_dir(pkgdir)
            .output()
            .map_err(|e| AURDeployError::ValidationFailed(e.to_string()))?;

        if output.status.success() {
            let srcinfo = output.stdout;
            std::fs::write(pkgdir.join(".SRCINFO"), srcinfo)?;
            info!("Generated .SRCINFO");
            Ok(())
        } else {
            Err(AURDeployError::ValidationFailed(
                String::from_utf8_lossy(&output.stderr).to_string()
            ))
        }
    }
}