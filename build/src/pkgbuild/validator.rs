//! PKGBUILD v2.2 validator
//!
//! Validates PKGBUILD structure and v2.2 compliance

use super::{Pkgbuild, PkgbuildError, Result};

/// Validator for PKGBUILD v2.2 compliance
pub struct PkgbuildValidator;

impl PkgbuildValidator {
    /// Validate a complete PKGBUILD
    pub fn validate(pkgbuild: &Pkgbuild) -> Result<()> {
        // Required fields
        Self::validate_required(pkgbuild)?;

        // Version format
        Self::validate_version(pkgbuild)?;

        // Arrays and counts
        Self::validate_arrays(pkgbuild)?;

        // Functions
        Self::validate_functions(pkgbuild)?;

        // Source/checksum consistency
        Self::validate_source_checksums(pkgbuild)?;

        // Dependencies
        Self::validate_dependencies(pkgbuild)?;

        Ok(())
    }

    /// Validate required fields
    fn validate_required(pkgbuild: &Pkgbuild) -> Result<()> {
        if pkgbuild.pkgname.is_empty() {
            return Err(PkgbuildError::MissingField {
                field: "pkgname".to_string(),
            });
        }

        if pkgbuild.pkgname.chars().any(|c| !c.is_alphanumeric() && c != '-' && c != '_') {
            return Err(PkgbuildError::InvalidValue {
                field: "pkgname".to_string(),
                value: pkgbuild.pkgname.clone(),
            });
        }

        if pkgbuild.pkgver.is_empty() {
            return Err(PkgbuildError::MissingField {
                field: "pkgver".to_string(),
            });
        }

        // pkgver should be a valid version string
        if !is_valid_version(&pkgbuild.pkgver) {
            return Err(PkgbuildError::InvalidValue {
                field: "pkgver".to_string(),
                value: pkgbuild.pkgver.clone(),
            });
        }

        Ok(())
    }

    /// Validate version format
    fn validate_version(pkgbuild: &Pkgbuild) -> Result<()> {
        // Check pkgrel is non-negative
        if pkgbuild.pkgrel == 0 {
            return Err(PkgbuildError::InvalidValue {
                field: "pkgrel".to_string(),
                value: pkgbuild.pkgrel.to_string(),
            });
        }

        // Check epoch if present
        if let Some(epoch) = pkgbuild.epoch {
            if epoch == 0 {
                return Err(PkgbuildError::InvalidValue {
                    field: "epoch".to_string(),
                    value: "0".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate array counts and formats
    fn validate_arrays(pkgbuild: &Pkgbuild) -> Result<()> {
        // Validate arch format
        for arch in &pkgbuild.arch {
            match arch.as_str() {
                "any" | "x86_64" | "i686" | "aarch64" | "armv7l" | "armv6h" | "armv5tel" => {}
                _ => {
                    tracing::warn!("Unknown architecture: {}", arch);
                }
            }
        }

        // Validate license
        for license in &pkgbuild.license {
            if license == "unknown" {
                continue;
            }
            // Check for common SPDX licenses
            let valid_licenses = [
                "MIT", "BSD", "GPL", "GPL2", "GPL3", "LGPL", "LGPL2", "LGPL3",
                "Apache", "MPL", "CC0", "CC-BY", "CC-BY-SA", "ISC", "Python",
            ];
            if !valid_licuses_contains(license, &valid_licenses) {
                tracing::warn!("Unknown license: {}", license);
            }
        }

        Ok(())
    }

    /// Validate function requirements
    fn validate_functions(pkgbuild: &Pkgbuild) -> Result<()> {
        // At least package() function is required
        if pkgbuild.package.is_empty() {
            // Check for legacy split package format
            if pkgbuild.pkgname.contains('-') && pkgbuild.package.is_empty() {
                return Err(PkgbuildError::Validation {
                    message: "Split package must have package() function".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Validate source/checksum consistency
    fn validate_source_checksums(pkgbuild: &Pkgbuild) -> Result<()> {
        let source_count = pkgbuild.source.len();
        let sha256_count = pkgbuild.sha256sums.len();
        let sha512_count = pkgbuild.sha512sums.len();
        let b2_count = pkgbuild.b2sums.len();

        // Check if any checksums are provided, they should match source count
        let has_checksums = sha256_count > 0 || sha512_count > 0 || b2_count > 0;

        if has_checksums {
            if sha256_count > 0 && sha256_count != source_count {
                return Err(PkgbuildError::ArrayCountMismatch {
                    expected: source_count,
                    actual: sha256_count,
                });
            }

            if sha512_count > 0 && sha512_count != source_count {
                return Err(PkgbuildError::ArrayCountMismatch {
                    expected: source_count,
                    actual: sha512_count,
                });
            }

            if b2_count > 0 && b2_count != source_count {
                return Err(PkgbuildError::ArrayCountMismatch {
                    expected: source_count,
                    actual: b2_count,
                });
            }
        }

        // Validate checksum format
        for sum in &pkgbuild.sha256sums {
            if sum != "SKIP" && sum.len() != 64 {
                return Err(PkgbuildError::InvalidValue {
                    field: "sha256sums".to_string(),
                    value: sum.clone(),
                });
            }
        }

        for sum in &pkgbuild.sha512sums {
            if sum != "SKIP" && sum.len() != 128 {
                return Err(PkgbuildError::InvalidValue {
                    field: "sha512sums".to_string(),
                    value: sum.clone(),
                });
            }
        }

        Ok(())
    }

    /// Validate dependency specifications
    fn validate_dependencies(pkgbuild: &Pkgbuild) -> Result<()> {
        // Check for circular dependencies (basic check)
        for dep in &pkgbuild.depends {
            if dep.starts_with(&format!("{}=", pkgbuild.pkgname)) {
                return Err(PkgbuildError::Validation {
                    message: format!("Package depends on itself with version: {}", dep),
                });
            }
        }

        for dep in &pkgbuild.provides {
            if dep.starts_with(&format!("{}=", pkgbuild.pkgname)) {
                return Err(PkgbuildError::Validation {
                    message: format!("Package provides itself with version: {}", dep),
                });
            }
        }

        // Check for conflict with provides
        for conflict in &pkgbuild.conflicts {
            for provide in &pkgbuild.provides {
                if conflict.starts_with(&provide[..provide.find('=').unwrap_or(provide.len())]) {
                    return Err(PkgbuildError::Validation {
                        message: format!("Package both provides and conflicts: {}", provide),
                    });
                }
            }
        }

        Ok(())
    }
}

/// Check if a version string is valid
fn is_valid_version(ver: &str) -> bool {
    if ver.is_empty() {
        return false;
    }

    let chars: Vec<char> = ver.chars().collect();

    // First character must be alphanumeric
    if !chars[0].is_alphanumeric() {
        return false;
    }

    // Check for valid characters
    for c in &chars {
        if !c.is_alphanumeric() && *c != '_' && *c != '.' && *c != '-' {
            return false;
        }
    }

    // Should not end with a dash
    if ver.ends_with('-') {
        return false;
    }

    true
}

/// Check if license list contains a valid license (fuzzy match)
fn valid_licenses_contains(license: &str, valid: &[&str]) -> bool {
    let upper_license = license.to_uppercase();
    let upper_valid: Vec<String> = valid.iter().map(|s| s.to_uppercase()).collect();

    // Exact match
    if upper_valid.contains(&upper_license) {
        return true;
    }

    // Check if it's a known SPDX license (flexible match)
    upper_license.starts_with("GPL")
        || upper_license.starts_with("LGPL")
        || upper_license.starts_with("BSD")
        || upper_license.starts_with("MIT")
        || upper_license.starts_with("APACHE")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_required() {
        let pkgbuild = Pkgbuild {
            pkgname: "test".to_string(),
            pkgver: "1.0".to_string(),
            ..Default::default()
        };
        assert!(PkgbuildValidator::validate(&pkgbuild).is_ok());

        let invalid = Pkgbuild {
            pkgname: "".to_string(),
            pkgver: "1.0".to_string(),
            ..Default::default()
        };
        assert!(PkgbuildValidator::validate(&invalid).is_err());
    }

    #[test]
    fn test_validate_version() {
        let pkgbuild = Pkgbuild {
            pkgname: "test".to_string(),
            pkgver: "1.0.0".to_string(),
            pkgrel: 0, // Invalid
            ..Default::default()
        };
        assert!(PkgbuildValidator::validate(&pkgbuild).is_err());
    }

    #[test]
    fn test_validate_source_checksums() {
        let pkgbuild = Pkgbuild {
            pkgname: "test".to_string(),
            pkgver: "1.0".to_string(),
            source: vec![super::super::Source {
                url: "https://example.com".to_string(),
                arch: None,
            }],
            sha256sums: vec!["SKIP".to_string()],
            ..Default::default()
        };
        assert!(PkgbuildValidator::validate(&pkgbuild).is_ok());

        // Mismatched counts
        let invalid = Pkgbuild {
            pkgname: "test".to_string(),
            pkgver: "1.0".to_string(),
            source: vec![
                super::super::Source { url: "https://a.com".to_string(), arch: None },
                super::super::Source { url: "https://b.com".to_string(), arch: None },
            ],
            sha256sums: vec!["SKIP".to_string()],
            ..Default::default()
        };
        assert!(PkgbuildValidator::validate(&invalid).is_err());
    }
}