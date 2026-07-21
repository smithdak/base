use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Serialize;

use crate::base_home;
use crate::canon::Canon;
use crate::cli::{PackArgs, PackCommand};
use crate::config::validate_id;
use crate::lock::{LockMode, RepositoryLock};
use crate::pack;

use super::print_json;

pub fn run(args: PackArgs, json: bool) -> Result<()> {
    match args.command {
        PackCommand::New { id } => new(&id, json),
        PackCommand::Check { path } => check(&path, json),
    }
}

#[derive(Debug, Serialize)]
struct NewReport {
    pack: String,
    root: String,
    files: Vec<String>,
    next_steps: Vec<String>,
}

fn new(id: &str, json: bool) -> Result<()> {
    validate_id(id, "pack")?;
    let home = base_home()?;
    let _lock = RepositoryLock::global(&home, LockMode::Exclusive)?;
    let destination = pack::library_root(&home, id);
    if destination.exists() {
        bail!(
            "pack `{id}` already exists at {}; choose another id or edit it in place",
            destination.display()
        );
    }

    let files = scaffold(id);
    let staged = destination.with_file_name(format!(
        ".base-new-{id}-{}-{}",
        std::process::id(),
        chrono::Utc::now().timestamp_millis()
    ));
    if let Err(error) = write_tree(&staged, &files) {
        let _ = fs::remove_dir_all(&staged);
        return Err(error);
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("cannot create pack library {}", parent.display()))?;
    }
    fs::rename(&staged, &destination).with_context(|| {
        format!(
            "cannot install scaffolded pack at {}",
            destination.display()
        )
    })?;

    let report = NewReport {
        pack: id.to_owned(),
        root: destination.display().to_string().replace('\\', "/"),
        files: files.keys().cloned().collect(),
        next_steps: vec![
            "author canonical definitions under the kind folders (authored rewrites, not copies)"
                .to_owned(),
            format!("validate with `base pack check {}`", report_root(&destination)),
            "then `base adopt` the pack into a project".to_owned(),
        ],
    };
    if json {
        print_json(&report)
    } else {
        println!("scaffolded pack `{}` at {}", report.pack, report.root);
        for step in &report.next_steps {
            println!("  - {step}");
        }
        Ok(())
    }
}

fn report_root(destination: &Path) -> String {
    destination.display().to_string().replace('\\', "/")
}

fn scaffold(id: &str) -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    files.insert(
        "pack.md".to_owned(),
        format!(
            "---\nid: {id}\nversion: 0.1.0\ndescription: TODO one-line description of the {id} pack.\n---\n\n# {id} pack\n\nTODO: inventory, provenance, and adoption notes.\n\nAuthor canonical definitions under `rules/`, `agents/`, `skills/`, `pipelines/`,\n`policies/`, `verifiers/`, and `knowledge/`. Keep this pack immutable per version.\n"
        ),
    );
    files.insert(
        "knowledge/overview.md".to_owned(),
        format!(
            "# {id} overview\n\nTODO: what this pack encodes and where it came from.\n"
        ),
    );
    files
}

fn write_tree(root: &Path, files: &BTreeMap<String, String>) -> Result<()> {
    // Create the standard kind folders so authoring has an obvious home, even
    // before any are filled.
    for kind in [
        "rules",
        "agents",
        "skills",
        "pipelines",
        "pipelines/stages",
        "policies",
        "verifiers",
        "knowledge",
    ] {
        let dir = root.join(kind.replace('/', std::path::MAIN_SEPARATOR_STR));
        fs::create_dir_all(&dir)
            .with_context(|| format!("cannot create {}", dir.display()))?;
    }
    for (relative, content) in files {
        let path = root.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("cannot create {}", parent.display()))?;
        }
        fs::write(&path, content).with_context(|| format!("cannot write {}", path.display()))?;
    }
    Ok(())
}

#[derive(Debug, Serialize)]
struct CheckReport {
    pack: String,
    version: String,
    root: String,
    files: usize,
    rules: usize,
    agents: usize,
    skills: usize,
    stages: usize,
    pipelines: usize,
    policies: usize,
    verifiers: usize,
    knowledge: usize,
}

fn check(path: &Path, json: bool) -> Result<()> {
    let manifest = pack::read_manifest(path)?;
    let files = pack::collect_files(path)?;
    let canon = Canon::load_pack_dir(path, &manifest.id)
        .with_context(|| format!("pack `{}` has invalid canonical content", manifest.id))?;

    let report = CheckReport {
        pack: manifest.id.clone(),
        version: manifest.version,
        root: path.display().to_string().replace('\\', "/"),
        files: files.len(),
        rules: canon.rules.len(),
        agents: canon.agents.len(),
        skills: canon.skills.len(),
        stages: canon.stages.len(),
        pipelines: canon.pipelines.len(),
        policies: canon.policies.len(),
        verifiers: canon.verifiers.len(),
        knowledge: canon.knowledge.len(),
    };
    if json {
        print_json(&report)
    } else {
        println!(
            "pack `{}` version {} is valid ({} files)",
            report.pack, report.version, report.files
        );
        println!(
            "  rules {}, agents {}, skills {}, stages {}, pipelines {}, policies {}, verifiers {}, knowledge {}",
            report.rules,
            report.agents,
            report.skills,
            report.stages,
            report.pipelines,
            report.policies,
            report.verifiers,
            report.knowledge,
        );
        Ok(())
    }
}
