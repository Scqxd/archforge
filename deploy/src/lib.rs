//! Deployment agents for AUR, Docker, Flatpak, and Nix

pub mod aur;

pub use aur::AURUploader;
pub use aur::AURDeployError;