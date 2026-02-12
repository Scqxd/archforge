//! PKGBUILD parsing and generation
//!
//! Supports PKGBUILD v2.2 format

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

pub use parser::parse_pkgbuild;
pub use generator::generate_pkgbuild;

mod parser;
mod generator;

/// Main PKGBUILD structure
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Pkgbuild {
    pub pkgname: String,
    pub pkgver: String,
    #[serde(default = "default_pkgrel")]
    pub pkgrel: u32,
    pub epoch: Option<u32>,
    pub pkgdesc: Option<String>,
    pub url: Option<String>,
    #[serde(default = "default_arch")]
    pub arch: Vec<String>,
    #[serde(default = "default_license")]
    pub license: Vec<String>,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub depends: Vec<String>,
    #[serde(default)]
    pub makedepends: Vec<String>,
    #[serde(default)]
    pub checkdepends: Vec<String>,
    #[serde(default)]
    pub optdepends: Vec<String>,
    #[serde(default)]
    pub provides: Vec<String>,
    #[serde(default)]
    pub conflicts: Vec<String>,
    #[serde(default)]
    pub replaces: Vec<String>,
    #[serde(default)]
    pub backup: Vec<String>,
    #[serde(default)]
    pub options: Vec<String>,
    pub install: Option<String>,
    pub changelog: Option<String>,
    #[serde(default)]
    pub source: Vec<Source>,
    #[serde(default)]
    pub sha256sums: Vec<String>,
    #[serde(default)]
    pub sha512sums: Vec<String>,
    #[serde(default)]
    pub b2sums: Vec<String>,
    #[serde(default)]
    pub validpgpkeys: Vec<String>,
    #[serde(default)]
    pub prepare: Vec<Function>,
    #[serde(default)]
    pub build: Vec<Function>,
    #[serde(default)]
    pub check: Vec<Function>,
    #[serde(default)]
    pub package: Vec<Function>,
}

fn default_pkgrel() -> u32 { 1 }
fn default_arch() -> Vec<String> { vec!["x86_64".to_string()] }
fn default_license() -> Vec<String> { vec!["unknown".to_string()] }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Source {
    pub url: String,
    #[serde(default)]
    pub arch: Option<String>,
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.url)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Function {
    pub name: Option<String>,
    pub body: String,
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{}() {{\n{}}}", name, self.body)
        } else {
            write!(f, "{{\n{}}}", self.body)
        }
    }
}

#[derive(Debug, Error)]
pub enum PkgbuildError {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Missing field: {0}")]
    MissingField(String),

    #[error("IO error: {source}")]
    Io { source: std::io::Error },
}

pub type Result<T> = std::result::Result<T, PkgbuildError>;

impl From<std::io::Error> for PkgbuildError {
    fn from(e: std::io::Error) -> Self {
        Self::Io { source: e }
    }
}