//! Language detection from package description
//!
//! Analyzes the description string to detect the programming language
//! and build system used by the project.

use super::TemplateKind;

/// Language detection patterns
const RUST_KEYWORDS: &[&str] = &["rust", "cargo", "rs"];
const GO_KEYWORDS: &[&str] = &["go", "golang", " gol "];
const PYTHON_KEYWORDS: &[&str] = &["python", "py", "pip", "pypi", "django", "flask", "fastapi"];
const CPP_KEYWORDS: &[&str] = &["c++", "cpp", "cxx", "c++", "cplusplus"];
const C_KEYWORDS: &[&str] = &["c", "c ", " c ", " c\n", "c.", "c,", "c-", "ansi-c", "plain c"];
const NODEJS_KEYWORDS: &[&str] = &["node", "nodejs", "npm", "javascript", "js ", " js", "express", "react", "vue"];
const HASKELL_KEYWORDS: &[&str] = &["haskell", "cabal", "hackage", "hs"];
const CMAKE_KEYWORDS: &[&str] = &["cmake"];
const MESON_KEYWORDS: &[&str] = &["meson", "ninja"];
const PERL_KEYWORDS: &[&str] = &["perl", "pl ", " cpan"];
const RUBY_KEYWORDS: &[&str] = &["ruby", "gem", "rails", "bundler"];
const DOTNET_KEYWORDS: &[&str] = &["dotnet", "c#", "csharp", ".net", "asp.net", "nuget"];
const JAVA_KEYWORDS: &[&str] = &["java", "maven", "gradle", "spring", "jvm", "jdk"];
const QT_KEYWORDS: &[&str] = &["qt", "qmake", "qt5", "qt6", "widgets"];

/// Detect the programming language from a package description
///
/// Uses keyword matching to identify the most likely language.
/// Priority is based on specificity of keywords (Qt, Meson first).
///
/// # Examples
///
/// ```
/// use archforge::templates::language::detect_language;
/// use archforge::templates::TemplateKind;
///
/// let kind = detect_language("rust http server library");
/// assert_eq!(kind, TemplateKind::Rust);
/// ```
pub fn detect_language(description: &str) -> TemplateKind {
    let desc_lower = description.to_lowercase();

    // Check for Qt first (very specific)
    if has_any_keyword(&desc_lower, QT_KEYWORDS) {
        return TemplateKind::Qt;
    }

    // Check for Meson (specific build system)
    if has_any_keyword(&desc_lower, MESON_KEYWORDS) {
        return TemplateKind::Meson;
    }

    // Check for CMake
    if has_any_keyword(&desc_lower, CMAKE_KEYWORDS) {
        return TemplateKind::CMake;
    }

    // Check for Haskell
    if has_any_keyword(&desc_lower, HASKELL_KEYWORDS) {
        return TemplateKind::Haskell;
    }

    // Check for Ruby
    if has_any_keyword(&desc_lower, RUBY_KEYWORDS) {
        return TemplateKind::Ruby;
    }

    // Check for Perl
    if has_any_keyword(&desc_lower, PERL_KEYWORDS) {
        return TemplateKind::Perl;
    }

    // Check for .NET
    if has_any_keyword(&desc_lower, DOTNET_KEYWORDS) {
        return TemplateKind::Dotnet;
    }

    // Check for Java
    if has_any_keyword(&desc_lower, JAVA_KEYWORDS) {
        return TemplateKind::Java;
    }

    // Check for Node.js/JavaScript
    if has_any_keyword(&desc_lower, NODEJS_KEYWORDS) {
        return TemplateKind::NodeJs;
    }

    // Check for Rust
    if has_any_keyword(&desc_lower, RUST_KEYWORDS) {
        return TemplateKind::Rust;
    }

    // Check for Python (before Go since "go" is a substring of some words)
    if has_any_keyword(&desc_lower, PYTHON_KEYWORDS) {
        return TemplateKind::Python;
    }

    // Check for Go
    if has_any_keyword(&desc_lower, GO_KEYWORDS) {
        return TemplateKind::Go;
    }

    // Check for C++ (before C to avoid matching "c" in "c++")
    if has_any_keyword(&desc_lower, CPP_KEYWORDS) {
        return TemplateKind::Cpp;
    }

    // Check for C
    if has_any_keyword(&desc_lower, C_KEYWORDS) {
        return TemplateKind::C;
    }

    // Default to generic C template
    TemplateKind::Generic
}

/// Check if the description contains any of the keywords as whole words
fn has_any_keyword(desc: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|kw| {
        // Check for whole word match
        let mut start = 0;
        while let Some(pos) = desc[start..].find(kw) {
            let abs_pos = start + pos;

            // Check character BEFORE keyword (should be non-alphanumeric or start of string)
            let before_ok = abs_pos == 0 ||
                !desc[(abs_pos - 1)..abs_pos].chars().next().is_some_and(|c| c.is_alphanumeric());

            // Check character AFTER keyword (should be non-alphanumeric or end of string)
            let after_pos = abs_pos + kw.len();
            let after_ok = after_pos >= desc.len() ||
                !desc[after_pos..].chars().next().is_some_and(|c| c.is_alphanumeric());

            if before_ok && after_ok {
                return true;
            }
            start = abs_pos + 1;
        }
        false
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_rust() {
        assert_eq!(detect_language("rust http server"), TemplateKind::Rust);
        assert_eq!(detect_language("cargo crate for parsing"), TemplateKind::Rust);
        assert_eq!(detect_language("rs library"), TemplateKind::Rust);
    }

    #[test]
    fn test_detect_go() {
        assert_eq!(detect_language("go web server"), TemplateKind::Go);
        assert_eq!(detect_language("golang api client"), TemplateKind::Go);
        assert_eq!(detect_language("go cli tool"), TemplateKind::Go);
    }

    #[test]
    fn test_detect_python() {
        assert_eq!(detect_language("python web scraper"), TemplateKind::Python);
        assert_eq!(detect_language("django app"), TemplateKind::Python);
        assert_eq!(detect_language("flask api"), TemplateKind::Python);
        assert_eq!(detect_language("pip package"), TemplateKind::Python);
        // Ensure "go" doesn't match inside "python"
        assert_eq!(detect_language("python go framework"), TemplateKind::Python);
    }

    #[test]
    fn test_detect_cpp() {
        assert_eq!(detect_language("c++ game engine"), TemplateKind::Cpp);
        assert_eq!(detect_language("cpp image processor"), TemplateKind::Cpp);
        assert_eq!(detect_language("c++ library"), TemplateKind::Cpp);
    }

    #[test]
    fn test_detect_c() {
        assert_eq!(detect_language("a c library"), TemplateKind::C);
        assert_eq!(detect_language("ansi-c utility"), TemplateKind::C);
        assert_eq!(detect_language(" plain c library"), TemplateKind::C);
    }

    #[test]
    fn test_detect_nodejs() {
        assert_eq!(detect_language("nodejs api server"), TemplateKind::NodeJs);
        assert_eq!(detect_language("javascript library"), TemplateKind::NodeJs);
        assert_eq!(detect_language("react component"), TemplateKind::NodeJs);
        assert_eq!(detect_language("npm package"), TemplateKind::NodeJs);
    }

    #[test]
    fn test_detect_cmake() {
        assert_eq!(detect_language("cmake project"), TemplateKind::CMake);
        assert_eq!(detect_language("cmake build system"), TemplateKind::CMake);
    }

    #[test]
    fn test_detect_meson() {
        assert_eq!(detect_language("meson build"), TemplateKind::Meson);
        assert_eq!(detect_language("meson project"), TemplateKind::Meson);
    }

    #[test]
    fn test_detect_qt() {
        assert_eq!(detect_language("qt5 application"), TemplateKind::Qt);
        assert_eq!(detect_language("qt gui app"), TemplateKind::Qt);
        assert_eq!(detect_language("qmake project"), TemplateKind::Qt);
    }

    #[test]
    fn test_detect_perl() {
        assert_eq!(detect_language("perl script"), TemplateKind::Perl);
        assert_eq!(detect_language("perl cpan module"), TemplateKind::Perl);
    }

    #[test]
    fn test_detect_ruby() {
        assert_eq!(detect_language("ruby gem"), TemplateKind::Ruby);
        assert_eq!(detect_language("rails app"), TemplateKind::Ruby);
        assert_eq!(detect_language("ruby library"), TemplateKind::Ruby);
    }

    #[test]
    fn test_detect_java() {
        assert_eq!(detect_language("java application"), TemplateKind::Java);
        assert_eq!(detect_language("spring boot app"), TemplateKind::Java);
        assert_eq!(detect_language("maven project"), TemplateKind::Java);
    }

    #[test]
    fn test_detect_haskell() {
        assert_eq!(detect_language("haskell library"), TemplateKind::Haskell);
        assert_eq!(detect_language("cabal package"), TemplateKind::Haskell);
    }

    #[test]
    fn test_detect_dotnet() {
        assert_eq!(detect_language("dotnet application"), TemplateKind::Dotnet);
        assert_eq!(detect_language("c# web api"), TemplateKind::Dotnet);
        assert_eq!(detect_language("asp.net core"), TemplateKind::Dotnet);
    }

    #[test]
    fn test_detect_unknown() {
        assert_eq!(detect_language("some random software"), TemplateKind::Generic);
        assert_eq!(detect_language("image converter"), TemplateKind::Generic);
    }
}