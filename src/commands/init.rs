use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Serialize;

use crate::base_home;
use crate::cli::InitArgs;
use crate::find_project_root;
use crate::lock::{LockMode, RepositoryLock};
use crate::templates;

use super::print_json;

#[derive(Debug, Serialize)]
struct InitReport {
    scope: &'static str,
    root: String,
    created: Vec<String>,
    replaced: Vec<String>,
    unchanged: Vec<String>,
}

#[derive(Debug, Default)]
pub(super) struct ScaffoldReport {
    pub created: Vec<String>,
    pub replaced: Vec<String>,
    pub unchanged: Vec<String>,
}

/// Write scaffold files under an already-held lock, refusing every collision
/// before creating anything so a failed scaffold never leaves partial output.
pub(super) fn scaffold(
    root: &Path,
    scope: &'static str,
    files: BTreeMap<String, String>,
    force: bool,
) -> Result<ScaffoldReport> {
    let mut report = ScaffoldReport::default();

    for (relative, content) in &files {
        let path = root.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
        if !path.exists() {
            continue;
        }
        let existing =
            fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
        let preserve_state = scope == "project" && relative == ".base/history.jsonl";
        let preserve_config = scope == "project" && relative == ".base/base.toml" && !force;
        if preserve_config {
            let config: crate::config::Config = toml::from_str(&existing)
                .with_context(|| format!("invalid TOML in {}", path.display()))?;
            config.validate()?;
        }
        if existing != *content && !preserve_state && !preserve_config && !force {
            bail!(
                "refusing to replace existing scaffold file {}; rerun with --force",
                path.display()
            );
        }
    }

    for (relative, content) in files {
        let path = root.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("cannot create {}", parent.display()))?;
        }
        if path.exists() {
            let existing = fs::read_to_string(&path)
                .with_context(|| format!("cannot read {}", path.display()))?;
            let preserve_state = scope == "project" && relative == ".base/history.jsonl";
            let preserve_config = scope == "project" && relative == ".base/base.toml" && !force;
            if existing == content || preserve_state || preserve_config {
                report.unchanged.push(relative);
                continue;
            }
            fs::write(&path, content)
                .with_context(|| format!("cannot replace {}", path.display()))?;
            report.replaced.push(relative);
        } else {
            fs::write(&path, content)
                .with_context(|| format!("cannot write {}", path.display()))?;
            report.created.push(relative);
        }
    }

    Ok(report)
}

pub fn run(start: &Path, args: InitArgs, json: bool) -> Result<()> {
    let (scope, root, files) = if args.packs_only {
        ("global-packs", base_home()?, templates::global_pack_files())
    } else if args.global {
        ("global", base_home()?, templates::global_files())
    } else if args.project {
        ("project", start.to_path_buf(), templates::project_files())
    } else {
        match find_project_root(start) {
            Ok(root) => ("project", root, templates::project_files()),
            Err(_) => ("global", base_home()?, templates::global_files()),
        }
    };

    fs::create_dir_all(&root)
        .with_context(|| format!("cannot create scaffold root {}", root.display()))?;
    let _lock = if scope == "project" {
        RepositoryLock::project(&root, LockMode::Exclusive)?
    } else {
        RepositoryLock::global(&root, LockMode::Exclusive)?
    };
    let scaffolded = scaffold(&root, scope, files, args.force)?;
    let report = InitReport {
        scope,
        root: root.display().to_string(),
        created: scaffolded.created,
        replaced: scaffolded.replaced,
        unchanged: scaffolded.unchanged,
    };

    if json {
        print_json(&report)
    } else {
        println!(
            "initialized {} base at {} ({} created, {} unchanged, {} replaced)",
            report.scope,
            report.root,
            report.created.len(),
            report.unchanged.len(),
            report.replaced.len()
        );
        if report.scope == "project" {
            println!("next: base check && base sync");
            println!("adopt the delivery operating model with: base adopt software-delivery");
            println!("or onboard end to end in one command with: base start");
        }
        Ok(())
    }
}
