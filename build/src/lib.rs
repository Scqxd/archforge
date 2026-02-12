//! Build engine and PKGBUILD parsing for ArchForge

pub mod pkgbuild;

// Re-exports
pub use pkgbuild::{Pkgbuild, parse_pkgbuild, generate_pkgbuild, PkgbuildError, Source, Function};