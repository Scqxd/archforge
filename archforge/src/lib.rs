//! ArchForge - AI-powered TUI for PKGBUILD generation and AUR management

pub mod ai;
pub mod cli;
#[cfg(feature = "tui")]
pub mod tui;

pub use ai::{AiProvider, ChutesClient};
pub use cli::Cli;

/// Version of ArchForge
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Homepage
pub const HOMEPAGE: &str = "https://github.com/archforge/archforge";