use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::Value;
use walkdir::WalkDir;

use crate::cli::LogArgs;

use super::print_json;

#[derive(Debug, Serialize)]
struct RunReport {
    slug: String,
    directory: String,
    files: Vec<String>,
    history: Vec<Value>,
}

pub fn run(project_root: &Path, args: LogArgs, json: bool) -> Result<()> {
    let history = load_history(project_root)?;
    match args.slug {
        None => list(history, json),
        Some(slug) => show(project_root, &slug, history, json),
    }
}

fn list(history: Vec<Value>, json: bool) -> Result<()> {
    if json {
        return print_json(&history);
    }
    if history.is_empty() {
        println!("no recorded runs");
        return Ok(());
    }
    println!(
        "{:<24} {:<12} {:<12} {:<10} OUTCOME",
        "SLUG", "DATE", "PIPELINE", "HARNESS"
    );
    for entry in history {
        println!(
            "{:<24} {:<12} {:<12} {:<10} {}",
            field(&entry, "slug"),
            field(&entry, "date"),
            field(&entry, "pipeline"),
            field(&entry, "harness"),
            field(&entry, "outcome")
        );
    }
    Ok(())
}

fn show(project_root: &Path, slug: &str, history: Vec<Value>, json: bool) -> Result<()> {
    if slug.contains('/') || slug.contains('\\') || slug == "." || slug == ".." {
        bail!("invalid run slug `{slug}`");
    }
    let runs_root = project_root.join(".base/runs");
    let exact = runs_root.join(slug);
    let directory = if exact.is_dir() {
        exact
    } else {
        let mut matches = fs::read_dir(&runs_root)
            .with_context(|| format!("cannot read {}", runs_root.display()))?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| {
                path.is_dir()
                    && path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.ends_with(slug))
            });
        let first = matches
            .next()
            .with_context(|| format!("run `{slug}` not found"))?;
        if matches.next().is_some() {
            bail!("run slug `{slug}` is ambiguous; use the full directory name");
        }
        first
    };
    let directory_name = directory
        .file_name()
        .and_then(|name| name.to_str())
        .context("run directory has no valid name")?
        .to_owned();
    let mut files = Vec::new();
    for entry in WalkDir::new(&directory).min_depth(1) {
        let entry = entry?;
        if entry.file_type().is_file() {
            files.push(
                entry
                    .path()
                    .strip_prefix(project_root)
                    .expect("run entry is below project")
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
    files.sort();
    let matching_history = history
        .into_iter()
        .filter(|entry| {
            entry
                .get("slug")
                .and_then(Value::as_str)
                .is_some_and(|value| value == slug || value == directory_name)
        })
        .collect();
    let report = RunReport {
        slug: directory_name,
        directory: directory
            .strip_prefix(project_root)
            .expect("run is below project")
            .to_string_lossy()
            .replace('\\', "/"),
        files,
        history: matching_history,
    };
    if json {
        print_json(&report)
    } else {
        println!("run: {}", report.slug);
        println!("directory: {}", report.directory);
        if report.files.is_empty() {
            println!("files: none");
        } else {
            println!("files:");
            for path in report.files {
                println!("  {path}");
            }
        }
        if !report.history.is_empty() {
            println!("history:");
            for entry in report.history {
                println!("  {}", serde_json::to_string(&entry)?);
            }
        }
        Ok(())
    }
}

fn load_history(project_root: &Path) -> Result<Vec<Value>> {
    let path = project_root.join(".base/history.jsonl");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let source = fs::read_to_string(&path)?;
    source
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            serde_json::from_str(line).with_context(|| {
                format!("invalid JSON on line {} of {}", index + 1, path.display())
            })
        })
        .collect()
}

fn field<'a>(value: &'a Value, name: &str) -> &'a str {
    value.get(name).and_then(Value::as_str).unwrap_or("-")
}
