use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Serialize;

use crate::base_home;
use crate::canon::Canon;
use crate::cli::StartArgs;
use crate::config::{Config, Target};
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
    native: NativeStage,
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

#[derive(Debug, Serialize)]
struct NativeStage {
    action: &'static str,
    found: Vec<String>,
    moved: Vec<String>,
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
    let found = detect_foreign_surfaces(&root, &config.targets)?;
    let native_stage = if found.is_empty() {
        NativeStage {
            action: "none",
            found,
            moved: Vec::new(),
        }
    } else if args.migrate_native {
        let moved = migrate_surfaces(&root, &found)?;
        NativeStage {
            action: "migrated",
            found,
            moved,
        }
    } else if args.force {
        for relative in &found {
            let path = root.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
            std::fs::remove_file(&path)
                .with_context(|| format!("cannot replace unowned surface {}", path.display()))?;
        }
        NativeStage {
            action: "replaced",
            found,
            moved: Vec::new(),
        }
    } else {
        bail!(
            "existing harness surface(s) {} would collide with generated output; rerun with --migrate-native to preserve them under .base/native/, move them yourself, or use --force to replace them",
            found.join(", ")
        );
    };

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
    match native_stage.action {
        "migrated" => next.push(
            "review the composed surfaces; the original bytes are preserved under .base/native/"
                .to_owned(),
        ),
        "replaced" => next.push(
            "existing harness surfaces were replaced by generated output; recover prior content from git history if needed"
                .to_owned(),
        ),
        _ => {}
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
        native: native_stage,
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
        if report.native.action == "migrated" {
            println!(
                "  native migration: moved {} into .base/native/",
                report.native.moved.join(", ")
            );
        }
        if report.native.action == "replaced" {
            println!(
                "  native surfaces replaced: {}",
                report.native.found.join(", ")
            );
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

/// Harness surfaces whose generated output would collide with pre-existing
/// files. Reuses the native-overlay table so migration and composition can
/// never disagree about the allowlist.
fn detect_foreign_surfaces(root: &Path, targets: &[Target]) -> Result<Vec<String>> {
    let owned: BTreeSet<String> = if Config::path(root).is_file() {
        Config::load(root)?.generated.keys().cloned().collect()
    } else {
        BTreeSet::new()
    };
    Ok(sync::NATIVE_OVERLAYS
        .iter()
        .filter(|(relative, target, _)| targets.contains(target) && !owned.contains(*relative))
        .map(|(relative, _, _)| relative.to_string())
        .filter(|relative| {
            root.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR))
                .is_file()
        })
        .collect())
}

/// Move recognized harness surfaces byte-preserving into `.base/native/`.
/// Existing overlays are never overwritten; the caller decides how to merge.
fn migrate_surfaces(root: &Path, found: &[String]) -> Result<Vec<String>> {
    let mut moved = Vec::new();
    for relative in found {
        let source = root.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
        let destination_relative = format!(".base/native/{relative}");
        let destination =
            root.join(destination_relative.replace('/', std::path::MAIN_SEPARATOR_STR));
        if destination.exists() {
            bail!(
                "native overlay {destination_relative} already exists; merge {} into it manually",
                relative
            );
        }
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("cannot create {}", parent.display()))?;
        }
        std::fs::rename(&source, &destination).with_context(|| {
            format!(
                "cannot move {} to {}",
                source.display(),
                destination.display()
            )
        })?;
        moved.push(relative.clone());
    }
    Ok(moved)
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
