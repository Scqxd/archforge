//! ArchForge - AI-powered TUI for PKGBUILD generation and AUR management

pub mod ai;
pub mod cli;
pub mod templates;
pub mod config;
#[cfg(feature = "tui")]
pub mod tui;

pub use ai::{AiProvider, ChutesClient};
pub use cli::Cli;
pub use templates::TemplateKind;
pub use config::Config;

/// Version of ArchForge
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Homepage
pub const HOMEPAGE: &str = "https://github.com/archforge/archforge";

/// Convert description to slug for package name
pub fn slugify(s: &str) -> String {
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