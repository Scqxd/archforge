//! PKGBUILD generator - converts Pkgbuild struct to string

use super::Pkgbuild;

/// Generate PKGBUILD string from Pkgbuild struct
pub fn generate_pkgbuild(pkgbuild: &Pkgbuild) -> String {
    let mut output = String::new();

    // Required fields
    output.push_str(&format!("pkgname={}\n", pkgbuild.pkgname));
    output.push_str(&format!("pkgver={}\n", pkgbuild.pkgver));

    // Optional fields with defaults
    if pkgbuild.pkgrel != 1 {
        output.push_str(&format!("pkgrel={}\n", pkgbuild.pkgrel));
    }

    if let Some(epoch) = pkgbuild.epoch {
        output.push_str(&format!("epoch={}\n", epoch));
    }

    if let Some(desc) = &pkgbuild.pkgdesc {
        output.push_str(&format!("pkgdesc={}\n", escape_shell(desc)));
    }

    if let Some(url) = &pkgbuild.url {
        output.push_str(&format!("url={}\n", escape_shell(url)));
    }

    // Arrays
    if !pkgbuild.arch.is_empty() {
        output.push_str(&format!("arch={}\n", format_array(&pkgbuild.arch)));
    }
    if !pkgbuild.license.is_empty() {
        output.push_str(&format!("license={}\n", format_array(&pkgbuild.license)));
    }

    if !pkgbuild.groups.is_empty() {
        output.push_str(&format!("groups={}\n", format_array(&pkgbuild.groups)));
    }
    if !pkgbuild.depends.is_empty() {
        output.push_str(&format!("depends={}\n", format_array(&pkgbuild.depends)));
    }
    if !pkgbuild.makedepends.is_empty() {
        output.push_str(&format!("makedepends={}\n", format_array(&pkgbuild.makedepends)));
    }
    if !pkgbuild.checkdepends.is_empty() {
        output.push_str(&format!("checkdepends={}\n", format_array(&pkgbuild.checkdepends)));
    }
    if !pkgbuild.optdepends.is_empty() {
        output.push_str(&format!("optdepends={}\n", format_array(&pkgbuild.optdepends)));
    }
    if !pkgbuild.provides.is_empty() {
        output.push_str(&format!("provides={}\n", format_array(&pkgbuild.provides)));
    }
    if !pkgbuild.conflicts.is_empty() {
        output.push_str(&format!("conflicts={}\n", format_array(&pkgbuild.conflicts)));
    }
    if !pkgbuild.replaces.is_empty() {
        output.push_str(&format!("replaces={}\n", format_array(&pkgbuild.replaces)));
    }
    if !pkgbuild.backup.is_empty() {
        output.push_str(&format!("backup={}\n", format_array(&pkgbuild.backup)));
    }
    if !pkgbuild.options.is_empty() {
        output.push_str(&format!("options={}\n", format_array(&pkgbuild.options)));
    }

    if let Some(install) = &pkgbuild.install {
        output.push_str(&format!("install={}\n", escape_shell(install)));
    }
    if let Some(changelog) = &pkgbuild.changelog {
        output.push_str(&format!("changelog={}\n", escape_shell(changelog)));
    }

    if !pkgbuild.source.is_empty() {
        output.push_str(&format!("source={}\n", format_source_array(&pkgbuild.source)));
    }
    if !pkgbuild.sha256sums.is_empty() {
        output.push_str(&format!("sha256sums={}\n", format_array(&pkgbuild.sha256sums)));
    }
    if !pkgbuild.sha512sums.is_empty() {
        output.push_str(&format!("sha512sums={}\n", format_array(&pkgbuild.sha512sums)));
    }
    if !pkgbuild.b2sums.is_empty() {
        output.push_str(&format!("b2sums={}\n", format_array(&pkgbuild.b2sums)));
    }
    if !pkgbuild.validpgpkeys.is_empty() {
        output.push_str(&format!("validpgpkeys={}\n", format_array(&pkgbuild.validpgpkeys)));
    }

    // Functions
    for func in &pkgbuild.prepare {
        output.push_str(&format_function(func));
    }
    for func in &pkgbuild.build {
        output.push_str(&format_function(func));
    }
    for func in &pkgbuild.check {
        output.push_str(&format_function(func));
    }
    for func in &pkgbuild.package {
        output.push_str(&format_function(func));
    }

    output
}

fn format_array(items: &[String]) -> String {
    if items.is_empty() {
        return "()".to_string();
    }
    let formatted: Vec<String> = items
        .iter()
        .map(|s| format!("'{}'", escape_shell(s)))
        .collect();
    format!("({})", formatted.join(" "))
}

fn format_source_array(sources: &[super::Source]) -> String {
    if sources.is_empty() {
        return "()".to_string();
    }
    let formatted: Vec<String> = sources
        .iter()
        .map(|s| format!("'{}'", s.url))
        .collect();
    format!("({})", formatted.join(" "))
}

fn format_function(func: &super::Function) -> String {
    let name = match &func.name {
        Some(n) => format!("{}()", n),
        None => String::from("{"),
    };
    format!("{} {{\n{}\n}}\n\n", name, indent(&func.body, 4))
}

fn indent(text: &str, n: usize) -> String {
    let indent = " ".repeat(n);
    text.lines()
        .map(|line| {
            if line.trim().is_empty() {
                line.to_string()
            } else {
                format!("{}{}", indent, line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn escape_shell(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' | '"' | '$' | '`' => result.push('\\'),
            _ => {}
        }
        result.push(c);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_simple() {
        let pkgbuild = Pkgbuild {
            pkgname: "test-package".to_string(),
            pkgver: "1.0.0".to_string(),
            pkgdesc: Some("A test package".to_string()),
            depends: vec!["glibc".to_string(), "gcc-libs".to_string()],
            ..Default::default()
        };

        let output = generate_pkgbuild(&pkgbuild);
        assert!(output.contains("pkgname=test-package"));
        assert!(output.contains("pkgver=1.0.0"));
        assert!(output.contains("depends=('glibc' 'gcc-libs')"));
    }
}