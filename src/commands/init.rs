use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Serialize;

use crate::base_home;
use crate::cli::InitArgs;
use crate::find_project_root;
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

pub fn run(start: &Path, args: InitArgs, json: bool) -> Result<()> {
    let (scope, root, files) = if args.global {
        ("global", base_home()?, templates::canon_files(""))
    } else if args.project {
        ("project", start.to_path_buf(), templates::project_files())
    } else {
        match find_project_root(start) {
            Ok(root) => ("project", root, templates::project_files()),
            Err(_) => ("global", base_home()?, templates::canon_files("")),
        }
    };

    fs::create_dir_all(&root)
        .with_context(|| format!("cannot create scaffold root {}", root.display()))?;
    let mut report = InitReport {
        scope,
        root: root.display().to_string(),
        created: Vec::new(),
        replaced: Vec::new(),
        unchanged: Vec::new(),
    };

    // Refuse all collisions before creating anything so a failed init never leaves a partial
    // scaffold. Existing history is state, not scaffold, and a configured manifest is preserved
    // by an ordinary idempotent init.
    for (relative, content) in &files {
        let path = root.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
        if !path.exists() {
            continue;
        }
        let existing =
            fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
        let preserve_state = scope == "project" && relative == ".base/history.jsonl";
        let preserve_config = scope == "project" && relative == ".base/base.toml" && !args.force;
        if preserve_config {
            let config: crate::config::Config = toml::from_str(&existing)
                .with_context(|| format!("invalid TOML in {}", path.display()))?;
            config.validate()?;
        }
        if existing != *content && !preserve_state && !preserve_config && !args.force {
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
            let preserve_config =
                scope == "project" && relative == ".base/base.toml" && !args.force;
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
        }
        Ok(())
    }
}
