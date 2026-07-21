use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::canon::split_frontmatter;
use crate::cli::{StateArgs, StateCommand};
use crate::config::Config;

use super::print_json;

const MAX_HANDOFF_BYTES: usize = 16 * 1024;

#[derive(Debug, Serialize)]
struct StateReport {
    current_work: Option<String>,
    item_path: Option<String>,
    active_run: Option<String>,
    handoff_path: Option<String>,
    handoff: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HandoffMeta {
    #[serde(rename = "work-item")]
    work_item: String,
    run: String,
}

struct ValidatedHandoff {
    source: String,
    run: String,
}

pub fn run(project_root: &Path, args: StateArgs, json: bool) -> Result<()> {
    Config::load(project_root)?;
    match args.command {
        StateCommand::Show => show(project_root, json, false),
        StateCommand::Context => show(project_root, json, true),
        StateCommand::Set { id } => set(project_root, &id, json),
        StateCommand::Clear => clear(project_root, json),
    }
}

fn show(project_root: &Path, json: bool, context: bool) -> Result<()> {
    let report = load(project_root)?;
    if json {
        return print_json(&report);
    }
    if context {
        println!(
            "base state: current work = {}",
            report.current_work.as_deref().unwrap_or("none")
        );
        if let Some(path) = &report.item_path {
            println!("work item: {path}");
        }
        if let Some(run) = &report.active_run {
            println!("active run: .base/runs/{run}");
        }
        if let (Some(path), Some(handoff)) = (&report.handoff_path, &report.handoff) {
            println!("handoff ({path}):");
            println!("{handoff}");
        } else {
            println!("handoff: none");
        }
        return Ok(());
    }
    println!(
        "current work: {}",
        report.current_work.as_deref().unwrap_or("none")
    );
    println!("item: {}", report.item_path.as_deref().unwrap_or("none"));
    println!(
        "active run: {}",
        report.active_run.as_deref().unwrap_or("none")
    );
    println!(
        "handoff: {}",
        report.handoff_path.as_deref().unwrap_or("none")
    );
    Ok(())
}

fn set(project_root: &Path, id: &str, json: bool) -> Result<()> {
    super::work::validate_work_id(id)?;
    let item = find_item(project_root, id)?;
    validate_handoff_for(project_root, Some(id))?;
    let state = project_root.join(".base/state");
    fs::create_dir_all(&state).with_context(|| format!("cannot create {}", state.display()))?;
    fs::write(state.join("current-work"), format!("{id}\n"))
        .context("cannot write .base/state/current-work")?;
    let report = load(project_root)?;
    if json {
        print_json(&report)
    } else {
        println!(
            "current work set to {id} ({})",
            item.strip_prefix(project_root).unwrap_or(&item).display()
        );
        Ok(())
    }
}

fn clear(project_root: &Path, json: bool) -> Result<()> {
    for path in [
        project_root.join(".base/state/current-work"),
        project_root.join(".base/state/handoff.md"),
    ] {
        match fs::remove_file(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| format!("cannot remove {}", path.display()));
            }
        }
    }
    let report = load(project_root)?;
    if json {
        print_json(&report)
    } else {
        println!("current work cleared");
        Ok(())
    }
}

fn load(project_root: &Path) -> Result<StateReport> {
    let pointer = project_root.join(".base/state/current-work");
    let current_work = if pointer.exists() {
        let value = fs::read_to_string(&pointer)
            .with_context(|| format!("cannot read {}", pointer.display()))?;
        let id = value.trim();
        super::work::validate_work_id(id)?;
        Some(id.to_owned())
    } else {
        None
    };
    let item_path = current_work
        .as_deref()
        .map(|id| find_item(project_root, id))
        .transpose()?
        .map(|path| relative(project_root, &path));

    let handoff = validate_handoff_for(project_root, current_work.as_deref())?;
    let active_run = handoff.as_ref().map(|handoff| handoff.run.clone());
    let handoff_path = handoff
        .as_ref()
        .map(|_| ".base/state/handoff.md".to_owned());
    Ok(StateReport {
        current_work,
        item_path,
        active_run,
        handoff_path,
        handoff: handoff.map(|handoff| handoff.source),
    })
}

fn validate_handoff_for(
    project_root: &Path,
    current_work: Option<&str>,
) -> Result<Option<ValidatedHandoff>> {
    let path = project_root.join(".base/state/handoff.md");
    if !path.exists() {
        return Ok(None);
    }
    let current_work = current_work.context(
        ".base/state/handoff.md exists without current work; set its work item or run `base state clear`",
    )?;
    let bytes = fs::read(&path).with_context(|| format!("cannot read {}", path.display()))?;
    if bytes.len() > MAX_HANDOFF_BYTES {
        bail!(
            ".base/state/handoff.md is {} bytes; keep the session handoff at or below {} bytes",
            bytes.len(),
            MAX_HANDOFF_BYTES
        );
    }
    let source = String::from_utf8(bytes).context(".base/state/handoff.md must be UTF-8")?;
    let (frontmatter, body) = split_frontmatter(&source)
        .context(".base/state/handoff.md must begin with YAML frontmatter")?;
    let meta: HandoffMeta = serde_yaml::from_str(frontmatter)
        .context("invalid YAML frontmatter in .base/state/handoff.md")?;
    super::work::validate_work_id(&meta.work_item)?;
    if meta.work_item != current_work {
        bail!(
            ".base/state/handoff.md belongs to {}, but current work is {}; update the handoff before switching work items",
            meta.work_item,
            current_work
        );
    }
    super::validate_run_slug(&meta.run)?;
    let run = project_root.join(".base/runs").join(&meta.run);
    if !run.is_dir() {
        bail!(
            ".base/state/handoff.md references missing run folder .base/runs/{}",
            meta.run
        );
    }
    let lines: Vec<&str> = body.lines().collect();
    let has_handoff = lines.iter().any(|line| line.trim() == "# Handoff");
    let next_action = lines
        .iter()
        .position(|line| line.trim() == "## Next action")
        .and_then(|index| {
            lines[index + 1..]
                .iter()
                .take_while(|line| !line.trim_start().starts_with('#'))
                .map(|line| line.trim())
                .find(|line| !line.is_empty())
        });
    if !has_handoff || next_action.is_none() {
        bail!(
            ".base/state/handoff.md must contain `# Handoff` and a non-empty `## Next action` section"
        );
    }
    Ok(Some(ValidatedHandoff {
        source: source.trim().to_owned(),
        run: meta.run,
    }))
}

fn find_item(project_root: &Path, id: &str) -> Result<PathBuf> {
    let work = project_root.join(".base/work");
    let mut matches = fs::read_dir(&work)
        .with_context(|| format!("cannot read {}", work.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name == id || name.starts_with(&format!("{id}-")))
                && path.join("item.md").is_file()
        });
    let item = matches
        .next()
        .with_context(|| format!("work item `{id}` not found under .base/work"))?;
    if matches.next().is_some() {
        bail!("work item id `{id}` is ambiguous under .base/work");
    }
    Ok(item.join("item.md"))
}

fn relative(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
