use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Serialize;

use crate::base_home;
use crate::canon::Canon;
use crate::cli::StartArgs;
use crate::config::Config;
use crate::lock::{LockMode, RepositoryLock};
use crate::templates;

use super::adopt;
use super::init;
use super::print_json;
use super::sync;
use super::work;

#[derive(Debug, Serialize)]
struct StartReport {
    root: String,
    global: GlobalStage,
    project: ProjectStage,
    pack: PackStage,
    canon: CanonStage,
    sync: SyncStage,
    next: Vec<String>,
}

#[derive(Debug, Serialize)]
struct GlobalStage {
    home: String,
    action: &'static str,
    files: usize,
}

#[derive(Debug, Serialize)]
struct ProjectStage {
    created: usize,
    replaced: usize,
    unchanged: usize,
}

#[derive(Debug, Serialize)]
struct PackStage {
    pack: String,
    version: Option<String>,
    action: &'static str,
}

#[derive(Debug, Serialize)]
struct CanonStage {
    rules: usize,
    agents: usize,
    skills: usize,
    stages: usize,
    pipelines: usize,
    policies: usize,
    verifiers: usize,
    knowledge: usize,
}

#[derive(Debug, Serialize)]
struct SyncStage {
    written: usize,
    unchanged: usize,
    removed: usize,
}

pub fn run(start: &Path, args: StartArgs, json: bool) -> Result<()> {
    let root = start.to_path_buf();
    std::fs::create_dir_all(&root)
        .with_context(|| format!("cannot create project root {}", root.display()))?;
    let _lock = RepositoryLock::project(&root, LockMode::Exclusive)?;
    let home = base_home()?;

    let global = ensure_global_library(&home, &args.pack, args.force)?;

    let scaffolded = init::scaffold(&root, "project", templates::project_files(), args.force)?;
    let project = ProjectStage {
        created: scaffolded.created.len(),
        replaced: scaffolded.replaced.len(),
        unchanged: scaffolded.unchanged.len(),
    };

    let pack_stage = if args.no_pack {
        PackStage {
            pack: String::new(),
            version: None,
            action: "skipped",
        }
    } else {
        let (action, version) = adopt::ensure(&root, &args.pack)?;
        PackStage {
            pack: args.pack.clone(),
            version: Some(version),
            action,
        }
    };

    let config = Config::load(&root)?;
    let canon = Canon::load(&home, &root, &config)?;
    work::validate(&root)?;
    let canon_stage = CanonStage {
        rules: canon.rules.len(),
        agents: canon.agents.len(),
        skills: canon.skills.len(),
        stages: canon.stages.len(),
        pipelines: canon.pipelines.len(),
        policies: canon.policies.len(),
        verifiers: canon.verifiers.len(),
        knowledge: canon.knowledge.len(),
    };

    let synced = sync::synchronize(&root, false, args.force)?;
    let sync_stage = SyncStage {
        written: synced.written.len(),
        unchanged: synced.unchanged.len(),
        removed: synced.removed.len(),
    };

    let mut next = Vec::new();
    if pack_stage.action == "adopted" {
        next.push(
            "review the adopted pack's policy and verifier commands before enabling generated hooks"
                .to_owned(),
        );
    }
    next.push(
        "invoke the delivery pipeline from your harness: /delivery <task> (Claude Code) or mention $delivery <task> (Codex, Copilot)"
            .to_owned(),
    );
    next.push("commit .base/ and the generated surfaces together".to_owned());

    let report = StartReport {
        root: root.display().to_string(),
        global,
        project,
        pack: pack_stage,
        canon: canon_stage,
        sync: sync_stage,
        next,
    };

    if json {
        print_json(&report)
    } else {
        println!("started base at {}", report.root);
        println!(
            "  global library {}: {}",
            report.global.home, report.global.action
        );
        println!(
            "  project scaffold: {} created, {} unchanged, {} replaced",
            report.project.created, report.project.unchanged, report.project.replaced
        );
        match (&report.pack.version, report.pack.action) {
            (Some(version), action) => println!("  pack {}: {version} {action}", report.pack.pack),
            (None, action) => println!("  pack adoption: {action}"),
        }
        println!(
            "  canon valid: {} rules, {} agents, {} skills, {} stages, {} pipelines, {} policies, {} verifiers, {} knowledge entries",
            report.canon.rules,
            report.canon.agents,
            report.canon.skills,
            report.canon.stages,
            report.canon.pipelines,
            report.canon.policies,
            report.canon.verifiers,
            report.canon.knowledge
        );
        println!(
            "  sync: {} written, {} unchanged, {} removed",
            report.sync.written, report.sync.unchanged, report.sync.removed
        );
        println!("next:");
        for hint in &report.next {
            println!("  - {hint}");
        }
        Ok(())
    }
}

/// Make the requested library pack available under BASE_HOME, scaffolding the
/// personal seed library on first use or only the missing pack afterwards.
fn ensure_global_library(home: &Path, pack: &str, force: bool) -> Result<GlobalStage> {
    let packs_root = home.join("canon").join("packs");
    if packs_root.join(pack).is_dir() {
        return Ok(GlobalStage {
            home: home.display().to_string(),
            action: "ready",
            files: 0,
        });
    }

    let all = templates::global_files();
    let prefix = format!("canon/packs/{pack}/");
    if !all.keys().any(|key| key.starts_with(&prefix)) {
        bail!(
            "no bundled library pack `{pack}`; available packs: {}",
            bundled_pack_ids(&all)
                .into_iter()
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    let seed_rule = home
        .join("canon")
        .join("rules")
        .join("working-agreements.md");
    let (scope, files): (&'static str, std::collections::BTreeMap<String, String>) =
        if seed_rule.is_file() {
            (
                "global-packs",
                all.into_iter()
                    .filter(|(key, _)| key.starts_with(&prefix))
                    .collect(),
            )
        } else {
            ("global", all)
        };

    let _lock = RepositoryLock::global(home, LockMode::Exclusive)?;
    let scaffolded = init::scaffold(home, scope, files, force)?;
    Ok(GlobalStage {
        home: home.display().to_string(),
        action: "initialized",
        files: scaffolded.created.len(),
    })
}

fn bundled_pack_ids(files: &std::collections::BTreeMap<String, String>) -> Vec<String> {
    let mut ids: Vec<String> = files
        .keys()
        .filter_map(|key| key.strip_prefix("canon/packs/"))
        .map(|rest| rest.split('/').next().unwrap_or(rest).to_owned())
        .collect();
    ids.sort();
    ids.dedup();
    ids
}
