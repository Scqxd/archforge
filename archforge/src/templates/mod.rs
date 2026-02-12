//! PKGBUILD template system for fallback generation
//!
//! Supports multiple programming languages and build systems:
//! - C/C++ (gcc/make)
//! - Go
//! - Python
//! - Rust (cargo)
//! - Node.js (npm)
//! - Haskell (cabal)
//! - CMake
//! - Meson
//! - Perl
//! - Ruby (gem)
//! - .NET (dotnet)
//! - Java (maven/gradle)
//! - Qt (qmake)

pub mod builder;
pub mod language;

use crate::templates::builder::TemplateBuilder;
use crate::templates::language::detect_language;

/// Supported template kinds for PKGBUILD generation
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateKind {
    /// C/C++ projects using gcc/make
    C,
    /// C++ specific projects
    Cpp,
    /// Go projects
    Go,
    /// Python projects
    Python,
    /// Rust projects using cargo
    Rust,
    /// Node.js projects using npm
    NodeJs,
    /// Haskell projects using cabal
    Haskell,
    /// CMake-based projects
    CMake,
    /// Meson + Ninja projects
    Meson,
    /// Perl projects
    Perl,
    /// Ruby gems
    Ruby,
    /// .NET projects
    Dotnet,
    /// Java projects (maven/gradle)
    Java,
    /// Qt projects (qmake)
    Qt,
    /// Generic/default fallback
    Generic,
}

impl TemplateKind {
    /// Create a template from description by detecting the language
    pub fn from_description(description: &str) -> Self {
        detect_language(description)
    }

    /// Generate a complete PKGBUILD for this template type
    pub fn generate_pkgbuild(&self, pkgname: &str, pkgver: &str, description: &str) -> String {
        let builder = TemplateBuilder::new(pkgname, pkgver, description);
        match self {
            Self::C => builder.build_c_template(),
            Self::Cpp => builder.build_cpp_template(),
            Self::Go => builder.build_go_template(),
            Self::Python => builder.build_python_template(),
            Self::Rust => builder.build_rust_template(),
            Self::NodeJs => builder.build_nodejs_template(),
            Self::Haskell => builder.build_haskell_template(),
            Self::CMake => builder.build_cmake_template(),
            Self::Meson => builder.build_meson_template(),
            Self::Perl => builder.build_perl_template(),
            Self::Ruby => builder.build_ruby_template(),
            Self::Dotnet => builder.build_dotnet_template(),
            Self::Java => builder.build_java_template(),
            Self::Qt => builder.build_qt_template(),
            Self::Generic => builder.build_generic_template(),
        }
    }

    /// Get makedepends for this template type
    pub fn makedepends(&self) -> &'static str {
        match self {
            Self::C | Self::Cpp => "('gcc' 'make')",
            Self::Go => "('go')",
            Self::Python => "('python' 'pip')",
            Self::Rust => "('cargo' 'rustc')",
            Self::NodeJs => "('nodejs' 'npm')",
            Self::Haskell => "('ghc' 'cabal')",
            Self::CMake => "('cmake' 'make' 'gcc')",
            Self::Meson => "('meson' 'ninja' 'gcc')",
            Self::Perl => "('perl' 'make')",
            Self::Ruby => "('ruby' 'ruby-bundler')",
            Self::Dotnet => "('dotnet-sdk')",
            Self::Java => "('jdk-openjdk' 'maven')",
            Self::Qt => "('qt5-base' 'make')",
            Self::Generic => "('gcc' 'make')",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_kind_from_description_rust() {
        let kind = TemplateKind::from_description("rust http server");
        assert_eq!(kind, TemplateKind::Rust);
    }

    #[test]
    fn test_template_kind_from_description_go() {
        let kind = TemplateKind::from_description("go web api");
        assert_eq!(kind, TemplateKind::Go);
    }

    #[test]
    fn test_template_kind_from_description_python() {
        let kind = TemplateKind::from_description("python web scraper");
        assert_eq!(kind, TemplateKind::Python);
    }

    #[test]
    fn test_template_kind_from_description_cpp() {
        let kind = TemplateKind::from_description("c++ image processor");
        assert_eq!(kind, TemplateKind::Cpp);
    }

    #[test]
    fn test_template_kind_from_description_nodejs() {
        let kind = TemplateKind::from_description("nodejs cli tool");
        assert_eq!(kind, TemplateKind::NodeJs);
    }

    #[test]
    fn test_template_kind_from_description_cmake() {
        let kind = TemplateKind::from_description("cmake project");
        assert_eq!(kind, TemplateKind::CMake);
    }

    #[test]
    fn test_template_kind_generic() {
        let kind = TemplateKind::from_description("some random software");
        assert_eq!(kind, TemplateKind::Generic);
    }

    #[test]
    fn test_generate_pkgbuild_rust() {
        let pkgbuild = TemplateKind::Rust.generate_pkgbuild("test-pkg", "1.0.0", "A test package");
        assert!(pkgbuild.contains("pkgname=test-pkg"));
        assert!(pkgbuild.contains("cargo build --release"));
    }

    #[test]
    fn test_generate_pkgbuild_go() {
        let pkgbuild = TemplateKind::Go.generate_pkgbuild("go-pkg", "1.0.0", "A go package");
        assert!(pkgbuild.contains("pkgname=go-pkg"));
        assert!(pkgbuild.contains("go build"));
    }
}