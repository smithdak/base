use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::cli::SyncArgs;
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
    let rendered = render::render(&canon, &config)?;
    let new_manifest: BTreeMap<String, String> = rendered
        .iter()
        .map(|(path, content)| (path.clone(), digest(content)))
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
                Some(content) if digest(&content) != *expected_hash => {
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
            let existing_hash = digest(existing);
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
            let existing_hash = digest(
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
        let existing_hash =
            digest(&fs::read(&path).with_context(|| format!("cannot read {}", path.display()))?);
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
        if path.exists() && digest(&fs::read(&path)?) != *recorded && !force {
            bail!(
                "stale generated file {} was hand-modified; use --force to remove it",
                path.display()
            );
        }
    }
    Ok(())
}

fn output_path(project_root: &Path, relative: &str) -> Result<PathBuf> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        bail!("unsafe generated output path `{relative}`");
    }
    Ok(project_root.join(relative_path))
}

fn digest(content: &[u8]) -> String {
    let bytes = Sha256::digest(content);
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("write to string");
    }
    output
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
}
