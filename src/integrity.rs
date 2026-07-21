use std::fmt::Write as _;
use std::path::{Component, Path};

use anyhow::{Result, bail};
use sha2::{Digest, Sha256};

pub fn digest(content: &[u8]) -> String {
    let bytes = Sha256::digest(content);
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("write to string");
    }
    output
}

pub fn validate_relative_path(value: &str, kind: &str) -> Result<()> {
    let path = Path::new(value);
    if value.trim().is_empty()
        || value != value.trim()
        || path.has_root()
        || value.contains(':')
        || value.contains('\\')
        || value.chars().any(char::is_control)
        || value.split('/').any(|component| {
            component.is_empty()
                || matches!(component, "." | "..")
                || !portable_component(component)
        })
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        bail!("{kind} must be a clean relative path, got `{value}`");
    }
    Ok(())
}

pub fn portable_component(component: &str) -> bool {
    if component.is_empty()
        || component.ends_with(['.', ' '])
        || component
            .chars()
            .any(|character| character.is_control() || "<>\"|?*:".contains(character))
    {
        return false;
    }
    let stem = component
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        && !(stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.as_bytes()[3].is_ascii_digit()
            && stem.as_bytes()[3] != b'0')
}

pub fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authored_paths_use_portable_slash_separators() {
        assert!(validate_relative_path("checks/api", "path").is_ok());
        for invalid in ["checks\\api", "checks//api", "./checks", "checks/../api"] {
            assert!(
                validate_relative_path(invalid, "path").is_err(),
                "{invalid}"
            );
        }
        for invalid in ["con", "docs/NUL.md", "tools/name.", "tools/a<b"] {
            assert!(
                validate_relative_path(invalid, "path").is_err(),
                "{invalid}"
            );
        }
    }
}
