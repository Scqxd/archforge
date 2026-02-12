//! Simple PKGBUILD parser

use super::{Pkgbuild, Function, Result};

/// Parse a PKGBUILD from string content
pub fn parse_pkgbuild(input: &str) -> Result<Pkgbuild> {
    let mut pkgbuild = Pkgbuild::default();
    let mut current_function: Option<Function> = None;

    for line in input.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Check for function definition
        if let Some(func_name) = parse_function_name(line) {
            // Save previous function if any
            if let Some(func) = current_function.take() {
                apply_function(&mut pkgbuild, &func);
            }
            current_function = Some(Function {
                name: Some(func_name.to_string()),
                body: String::new(),
            });
            continue;
        }

        // Check for function end
        if line == "}" {
            if let Some(func) = current_function.take() {
                apply_function(&mut pkgbuild, &func);
            }
            continue;
        }

        // Inside a function - add to body
        if let Some(ref mut func) = current_function {
            func.body.push_str(line);
            func.body.push('\n');
            continue;
        }

        // Parse variable assignment
        if let Some((name, value)) = parse_variable(line) {
            apply_variable(&mut pkgbuild, &name, &value);
        }
    }

    Ok(pkgbuild)
}

fn parse_function_name(line: &str) -> Option<&str> {
    // Check for "name() {" pattern
    if line.ends_with("() {") {
        let end = line.len() - 4; // Remove "() {"
        let name = &line[..end];
        if name.chars().next().map(|c| c.is_alphabetic() || c == '_').unwrap_or(false)
            && name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Some(name);
        }
    }
    // Also check for just "name()"
    if line.ends_with("()") {
        let end = line.len() - 2;
        let name = &line[..end];
        if name.chars().next().map(|c| c.is_alphabetic() || c == '_').unwrap_or(false)
            && name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Some(name);
        }
    }
    None
}

fn parse_variable(line: &str) -> Option<(&str, &str)> {
    if let Some(eq_pos) = line.find('=') {
        let name = line[..eq_pos].trim();
        let value = line[eq_pos + 1..].trim();

        // Validate name
        if name.is_empty() || !name.chars().next().map(|c| c.is_alphabetic() || c == '_').unwrap_or(false) {
            return None;
        }

        Some((name, value))
    } else {
        None
    }
}

fn apply_variable(pkgbuild: &mut Pkgbuild, name: &str, value: &str) {
    match name {
        "pkgname" => pkgbuild.pkgname = value.to_string(),
        "pkgver" => pkgbuild.pkgver = value.to_string(),
        "pkgrel" => pkgbuild.pkgrel = value.parse().unwrap_or(1),
        "epoch" => pkgbuild.epoch = Some(value.parse().unwrap_or(0)),
        "pkgdesc" => pkgbuild.pkgdesc = Some(value.to_string()),
        "url" => pkgbuild.url = Some(value.to_string()),
        "arch" => pkgbuild.arch = parse_string_array(value),
        "license" => pkgbuild.license = parse_string_array(value),
        "groups" => pkgbuild.groups = parse_string_array(value),
        "depends" => pkgbuild.depends = parse_string_array(value),
        "makedepends" => pkgbuild.makedepends = parse_string_array(value),
        "checkdepends" => pkgbuild.checkdepends = parse_string_array(value),
        "optdepends" => pkgbuild.optdepends = parse_string_array(value),
        "provides" => pkgbuild.provides = parse_string_array(value),
        "conflicts" => pkgbuild.conflicts = parse_string_array(value),
        "replaces" => pkgbuild.replaces = parse_string_array(value),
        "backup" => pkgbuild.backup = parse_string_array(value),
        "install" => pkgbuild.install = Some(value.to_string()),
        "changelog" => pkgbuild.changelog = Some(value.to_string()),
        "source" => pkgbuild.source = parse_source_array(value),
        "sha256sums" => pkgbuild.sha256sums = parse_string_array(value),
        "sha512sums" => pkgbuild.sha512sums = parse_string_array(value),
        "b2sums" => pkgbuild.b2sums = parse_string_array(value),
        "validpgpkeys" => pkgbuild.validpgpkeys = parse_string_array(value),
        _ => {} // Unknown variable, skip
    }
}

fn apply_function(pkgbuild: &mut Pkgbuild, func: &Function) {
    match func.name.as_deref() {
        Some("prepare") => pkgbuild.prepare.push(func.clone()),
        Some("build") => pkgbuild.build.push(func.clone()),
        Some("check") => pkgbuild.check.push(func.clone()),
        Some("package") => pkgbuild.package.push(func.clone()),
        _ => {} // Unknown function, skip
    }
}

fn parse_string_array(value: &str) -> Vec<String> {
    let value = value.trim();
    if value.starts_with('(') && value.ends_with(')') {
        let inner = &value[1..value.len()-1];
        inner.split_whitespace()
            .map(|s| s.trim_matches('\'').trim_matches('"').to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        vec![value.to_string()]
    }
}

fn parse_source_array(value: &str) -> Vec<super::Source> {
    let value = value.trim();
    if value.starts_with('(') && value.ends_with(')') {
        let inner = &value[1..value.len()-1];
        inner.split_whitespace()
            .map(|s| {
                let s = s.trim_matches('\'').trim_matches('"');
                super::Source {
                    url: s.to_string(),
                    arch: None,
                }
            })
            .filter(|s| !s.url.is_empty())
            .collect()
    } else {
        vec![super::Source {
            url: value.to_string(),
            arch: None,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_PKGBUILD: &str = r#"pkgname=example-package
pkgver=1.0.0
pkgrel=1
pkgdesc="An example package"
arch=('x86_64')
license=('MIT')
depends=('glibc' 'gcc-libs')

build() {
    cargo build --release
}

package() {
    install -Dm755 example "$pkgdir/usr/bin/example"
}
"#;

    #[test]
    fn test_parse_simple() {
        let pkgbuild = parse_pkgbuild(SAMPLE_PKGBUILD).unwrap();
        assert_eq!(pkgbuild.pkgname, "example-package");
        assert_eq!(pkgbuild.pkgver, "1.0.0");
        assert_eq!(pkgbuild.depends, vec!["glibc", "gcc-libs"]);
    }

    #[test]
    fn test_parse_functions() {
        let pkgbuild = parse_pkgbuild(SAMPLE_PKGBUILD).unwrap();
        assert_eq!(pkgbuild.build.len(), 1);
        assert_eq!(pkgbuild.package.len(), 1);
    }
}