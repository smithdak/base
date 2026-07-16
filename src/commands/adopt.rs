use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Serialize;

use crate::base_home;
use crate::cli::AdoptArgs;
use crate::config::Config;

use super::print_json;

#[derive(Debug, Serialize)]
struct AdoptReport {
    pack: String,
    root: String,
    copied: Vec<String>,
    follow_ups: Vec<String>,
}

pub fn run(project_root: &Path, args: AdoptArgs, json: bool) -> Result<()> {
    Config::load(project_root)?;

    let packs_root = base_home()?.join("canon").join("packs");
    let pack_root = packs_root.join(&args.pack);
    if !pack_root.is_dir() {
        bail!(
            "no pack `{}` in {}; {}",
            args.pack,
            packs_root.display(),
            available_packs(&packs_root)
        );
    }

    let mut files = Vec::new();
    collect_markdown(&pack_root, "", &mut files)?;
    files.retain(|relative| relative != "pack.md");
    files.sort();
    if files.is_empty() {
        bail!("pack `{}` has no canon files to adopt", args.pack);
    }

    // Refuse all collisions before copying anything so a failed adopt never leaves a
    // partial copy; adoption stays one visible, whole-pack change in the project's history.
    let canon_root = project_root.join(".base").join("canon");
    let conflicts: Vec<String> = files
        .iter()
        .filter(|relative| canon_root.join(native(relative)).exists())
        .map(|relative| format!(".base/canon/{relative}"))
        .collect();
    if !conflicts.is_empty() {
        bail!(
            "refusing to overwrite existing canon files: {}",
            conflicts.join(", ")
        );
    }

    for relative in &files {
        let source = pack_root.join(native(relative));
        let destination = canon_root.join(native(relative));
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("cannot create {}", parent.display()))?;
        }
        fs::copy(&source, &destination).with_context(|| {
            format!(
                "cannot copy {} to {}",
                source.display(),
                destination.display()
            )
        })?;
    }

    let mut follow_ups = Vec::new();
    let knowledge: Vec<&String> = files
        .iter()
        .filter(|relative| relative.starts_with("knowledge/"))
        .collect();
    if !knowledge.is_empty() {
        follow_ups.push(format!(
            "add a routing line to .base/canon/knowledge/INDEX.md for each of: {}",
            knowledge
                .iter()
                .map(|relative| relative.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    follow_ups.push("run `base sync`".to_owned());
    follow_ups.push("commit the copied canon and regenerated surfaces together".to_owned());
    if pack_root.join("pack.md").is_file() {
        follow_ups.push(format!(
            "see {} for pack-specific adoption notes",
            pack_root.join("pack.md").display()
        ));
    }

    let report = AdoptReport {
        pack: args.pack,
        root: project_root.display().to_string(),
        copied: files,
        follow_ups,
    };
    if json {
        print_json(&report)
    } else {
        println!(
            "adopted pack `{}` ({} files copied into .base/canon)",
            report.pack,
            report.copied.len()
        );
        for relative in &report.copied {
            println!("  .base/canon/{relative}");
        }
        println!("next:");
        for follow_up in &report.follow_ups {
            println!("  - {follow_up}");
        }
        Ok(())
    }
}

fn collect_markdown(directory: &Path, prefix: &str, files: &mut Vec<String>) -> Result<()> {
    let entries =
        fs::read_dir(directory).with_context(|| format!("cannot read {}", directory.display()))?;
    for entry in entries {
        let entry = entry.with_context(|| format!("cannot read {}", directory.display()))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();
        if path.is_dir() {
            collect_markdown(&path, &format!("{prefix}{name}/"), files)?;
        } else if path.extension().is_some_and(|extension| extension == "md") {
            files.push(format!("{prefix}{name}"));
        }
    }
    Ok(())
}

fn available_packs(packs_root: &Path) -> String {
    let mut packs: Vec<String> = fs::read_dir(packs_root)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    packs.sort();
    if packs.is_empty() {
        "no packs installed".to_owned()
    } else {
        format!("available packs: {}", packs.join(", "))
    }
}

fn native(relative: &str) -> String {
    relative.replace('/', std::path::MAIN_SEPARATOR_STR)
}
