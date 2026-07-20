use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::{Map, Value};

use crate::cli::SyncArgs;
use crate::config::{Config, Target};
use crate::integrity::digest;
use crate::render;

use super::{load_project, print_json};

#[derive(Debug, Serialize)]
struct SyncReport {
    mode: &'static str,
    written: Vec<String>,
    unchanged: Vec<String>,
    removed: Vec<String>,
    drift: Vec<String>,
}

pub fn run(project_root: &Path, args: SyncArgs, json: bool) -> Result<()> {
    if args.check && args.force {
        bail!("--check and --force cannot be used together");
    }
    let (mut config, canon) = load_project(project_root)?;
    let mut rendered = render::render(&canon, &config)?;
    apply_native_overlays(project_root, &config, &mut rendered)?;
    let new_manifest: BTreeMap<String, String> = rendered
        .iter()
        .map(|(path, content)| (path.clone(), generated_digest(content)))
        .collect();

    let mut report = SyncReport {
        mode: if args.check { "check" } else { "write" },
        written: Vec::new(),
        unchanged: Vec::new(),
        removed: Vec::new(),
        drift: Vec::new(),
    };

    if !args.check {
        preflight(
            project_root,
            &config.generated,
            &rendered,
            &new_manifest,
            args.force,
        )?;
    }

    for (relative, expected_content) in &rendered {
        let path = output_path(project_root, relative)?;
        let existing = fs::read(&path).ok();
        let old_hash = config.generated.get(relative);
        let expected_hash = new_manifest
            .get(relative)
            .expect("manifest built from output");

        if args.check {
            match existing {
                None => report.drift.push(format!("missing {relative}")),
                Some(content) if generated_digest(&content) != *expected_hash => {
                    report.drift.push(format!("content differs {relative}"));
                }
                Some(_) if old_hash != Some(expected_hash) => {
                    report
                        .drift
                        .push(format!("manifest hash differs {relative}"));
                }
                Some(_) => report.unchanged.push(relative.clone()),
            }
            continue;
        }

        if let Some(existing) = &existing {
            let existing_hash = generated_digest(existing);
            match old_hash {
                Some(recorded) if &existing_hash != recorded && !args.force => bail!(
                    "generated file {} was hand-modified; use `base sync --force` to replace it",
                    path.display()
                ),
                None if existing_hash != *expected_hash && !args.force => bail!(
                    "refusing to overwrite unowned file {}; move it or use `base sync --force`",
                    path.display()
                ),
                _ => {}
            }
            if existing_hash == *expected_hash {
                report.unchanged.push(relative.clone());
                continue;
            }
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("cannot create {}", parent.display()))?;
        }
        fs::write(&path, expected_content)
            .with_context(|| format!("cannot write generated file {}", path.display()))?;
        report.written.push(relative.clone());
    }

    for (relative, old_hash) in &config.generated {
        if rendered.contains_key(relative) {
            continue;
        }
        if args.check {
            report
                .drift
                .push(format!("stale manifest entry {relative}"));
            continue;
        }
        let path = output_path(project_root, relative)?;
        if path.exists() {
            let existing_hash = generated_digest(
                &fs::read(&path).with_context(|| format!("cannot read {}", path.display()))?,
            );
            if &existing_hash != old_hash && !args.force {
                bail!(
                    "stale generated file {} was hand-modified; use --force to remove it",
                    path.display()
                );
            }
            fs::remove_file(&path)
                .with_context(|| format!("cannot remove stale output {}", path.display()))?;
            remove_empty_parents(path.parent(), project_root)?;
        }
        report.removed.push(relative.clone());
    }

    if args.check {
        if !report.drift.is_empty() {
            if json {
                print_json(&report)?;
            }
            bail!(
                "generated output is out of sync: {}",
                report.drift.join(", ")
            );
        }
    } else {
        config.generated = new_manifest;
        config.save(project_root)?;
    }

    if json {
        print_json(&report)
    } else if args.check {
        println!(
            "sync check passed ({} generated files)",
            report.unchanged.len()
        );
        Ok(())
    } else {
        println!(
            "synced {} files ({} written, {} unchanged, {} removed)",
            report.written.len() + report.unchanged.len(),
            report.written.len(),
            report.unchanged.len(),
            report.removed.len()
        );
        Ok(())
    }
}

const NATIVE_OVERLAYS: [(&str, Target, bool); 6] = [
    ("CLAUDE.md", Target::Claude, false),
    (".claude/settings.json", Target::Claude, true),
    ("AGENTS.md", Target::Codex, false),
    (".codex/hooks.json", Target::Codex, true),
    (".github/copilot-instructions.md", Target::Copilot, false),
    (".github/hooks/base.json", Target::Copilot, true),
];

fn apply_native_overlays(
    project_root: &Path,
    config: &Config,
    rendered: &mut render::RenderedFiles,
) -> Result<()> {
    for (relative, target, json) in NATIVE_OVERLAYS {
        let overlay_relative = format!(".base/native/{relative}");
        let overlay_path = output_path(project_root, &overlay_relative)?;
        if !overlay_path.is_file() {
            continue;
        }
        if !config.targets.contains(&target) {
            bail!("native overlay `{overlay_relative}` targets disabled harness `{target}`");
        }
        let overlay = fs::read(&overlay_path)
            .with_context(|| format!("cannot read native overlay {}", overlay_path.display()))?;
        if json {
            let overlay_value: Value = serde_json::from_slice(&overlay)
                .with_context(|| format!("invalid JSON in native overlay `{overlay_relative}`"))?;
            if !overlay_value.is_object() {
                bail!("native JSON overlay `{overlay_relative}` must contain an object");
            }
            if overlay_value
                .get("disableAllHooks")
                .and_then(Value::as_bool)
                == Some(true)
            {
                bail!("native JSON overlay `{overlay_relative}` cannot disable Base-owned hooks");
            }
            let mut base_value = match rendered.get(relative) {
                Some(base) => serde_json::from_slice(base)
                    .with_context(|| format!("invalid generated JSON for `{relative}`"))?,
                None => Value::Object(Map::new()),
            };
            merge_json_base_wins(&mut base_value, overlay_value);
            let mut source = serde_json::to_vec_pretty(&base_value).with_context(|| {
                format!("cannot serialize composed native surface `{relative}`")
            })?;
            source.push(b'\n');
            rendered.insert(relative.to_owned(), source);
        } else {
            let base = rendered.get(relative).with_context(|| {
                format!("native Markdown overlay `{overlay_relative}` has no generated target")
            })?;
            let base = std::str::from_utf8(base)
                .with_context(|| format!("generated Markdown `{relative}` is not UTF-8"))?;
            let overlay = std::str::from_utf8(&overlay).with_context(|| {
                format!("native Markdown overlay `{overlay_relative}` is not UTF-8")
            })?;
            let composed = format!(
                "{}\n<!-- project native supplement from {} -->\n\n{}\n",
                base.trim_end(),
                overlay_relative,
                overlay.trim()
            );
            rendered.insert(relative.to_owned(), composed.into_bytes());
        }
    }
    Ok(())
}

fn merge_json_base_wins(base: &mut Value, overlay: Value) {
    match (base, overlay) {
        (Value::Object(base), Value::Object(overlay)) => {
            for (key, overlay_value) in overlay {
                match base.get_mut(&key) {
                    Some(base_value) => merge_json_base_wins(base_value, overlay_value),
                    None => {
                        base.insert(key, overlay_value);
                    }
                }
            }
        }
        (Value::Array(base), Value::Array(mut overlay)) => {
            overlay.append(base);
            *base = overlay;
        }
        // Canonical Base values win scalar/type conflicts so a native overlay
        // cannot remove required hook or policy fields.
        _ => {}
    }
}

fn preflight(
    project_root: &Path,
    old_manifest: &BTreeMap<String, String>,
    rendered: &render::RenderedFiles,
    new_manifest: &BTreeMap<String, String>,
    force: bool,
) -> Result<()> {
    for (relative, expected_hash) in new_manifest {
        let path = output_path(project_root, relative)?;
        if !path.exists() {
            continue;
        }
        let existing_hash = generated_digest(
            &fs::read(&path).with_context(|| format!("cannot read {}", path.display()))?,
        );
        match old_manifest.get(relative) {
            Some(recorded) if &existing_hash != recorded && !force => bail!(
                "generated file {} was hand-modified; use `base sync --force` to replace it",
                path.display()
            ),
            None if &existing_hash != expected_hash && !force => bail!(
                "refusing to overwrite unowned file {}; move it or use `base sync --force`",
                path.display()
            ),
            _ => {}
        }
    }
    for (relative, recorded) in old_manifest {
        if rendered.contains_key(relative) {
            continue;
        }
        let path = output_path(project_root, relative)?;
        if path.exists() && generated_digest(&fs::read(&path)?) != *recorded && !force {
            bail!(
                "stale generated file {} was hand-modified; use --force to remove it",
                path.display()
            );
        }
    }
    Ok(())
}

fn generated_digest(content: &[u8]) -> String {
    let Ok(text) = std::str::from_utf8(content) else {
        return digest(content);
    };
    if !text.contains("\r\n") {
        return digest(content);
    }
    digest(text.replace("\r\n", "\n").as_bytes())
}

fn output_path(project_root: &Path, relative: &str) -> Result<PathBuf> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute() {
        bail!("unsafe generated output path `{relative}`");
    }
    let mut output = project_root.to_path_buf();
    for component in relative_path.components() {
        let std::path::Component::Normal(component) = component else {
            bail!("unsafe generated output path `{relative}`");
        };
        output.push(component);
        match fs::symlink_metadata(&output) {
            Ok(metadata) if is_link_or_reparse_point(&metadata) => bail!(
                "generated output path `{relative}` traverses link or reparse point {}",
                output.display()
            ),
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("cannot inspect generated output path {}", output.display())
                });
            }
        }
    }
    Ok(output)
}

#[cfg(windows)]
fn is_link_or_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_link_or_reparse_point(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

fn remove_empty_parents(mut directory: Option<&Path>, project_root: &Path) -> Result<()> {
    while let Some(path) = directory {
        if path == project_root || !path.starts_with(project_root) {
            break;
        }
        match fs::remove_dir(path) {
            Ok(()) => directory = path.parent(),
            Err(error) if error.kind() == std::io::ErrorKind::DirectoryNotEmpty => break,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                directory = path.parent();
            }
            Err(error) => {
                return Err(error).with_context(|| format!("cannot remove {}", path.display()));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_is_stable() {
        assert_eq!(
            digest(b"base"),
            "cae662172fd450bb0cd710a769079c05bfc5d8e35efa6576edc7d0377afdd4a2"
        );
    }

    #[test]
    fn generated_text_hashes_are_crlf_insensitive_but_binary_hashes_are_exact() {
        assert_eq!(
            generated_digest(b"one\ntwo\n"),
            generated_digest(b"one\r\ntwo\r\n")
        );
        assert_ne!(
            generated_digest(b"one\ntwo\n"),
            generated_digest(b"one\r\ntoo\r\n")
        );
        assert_ne!(
            generated_digest(&[0xff, b'\n']),
            generated_digest(&[0xff, b'\r', b'\n'])
        );
    }

    #[cfg(unix)]
    #[test]
    fn output_paths_refuse_symlink_components() {
        use std::os::unix::fs::symlink;

        let project = tempfile::TempDir::new().unwrap();
        let outside = tempfile::TempDir::new().unwrap();
        symlink(outside.path(), project.path().join(".claude")).unwrap();
        let error = output_path(project.path(), ".claude/settings.json").unwrap_err();
        assert!(error.to_string().contains("traverses link"));
        assert!(!outside.path().join("settings.json").exists());
    }
}
