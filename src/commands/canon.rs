use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Serialize;

use crate::base_home;
use crate::cli::{CanonCommand, CanonKind};
use crate::config::validate_id;

use super::check;
use super::print_json;

#[derive(Debug, Serialize)]
struct NewReport {
    target: String,
    kind: String,
    id: String,
    path: String,
    next: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ListReport {
    count: usize,
    definitions: Vec<DefinitionRow>,
}

#[derive(Debug, Serialize)]
struct DefinitionRow {
    kind: &'static str,
    id: String,
    source: String,
}

pub fn run(project_root: &Path, args: crate::cli::CanonArgs, json: bool) -> Result<()> {
    match args.command {
        CanonCommand::New { kind, id, pack } => new(project_root, kind, &id, pack, json),
        CanonCommand::List { kind } => list(project_root, kind, json),
    }
}

fn new(
    project_root: &Path,
    kind: CanonKind,
    id: &str,
    pack: Option<String>,
    json: bool,
) -> Result<()> {
    validate_id(id, kind.name())?;

    let (root_label, canon_root) = match &pack {
        Some(pack_id) => {
            validate_id(pack_id, "pack")?;
            let home = base_home()?;
            let pack_root = home.join("canon").join("packs").join(pack_id);
            if !pack_root.is_dir() {
                bail!(
                    "no library pack `{pack_id}` at {}; create one with `base pack new {pack_id}`",
                    pack_root.display()
                );
            }
            (format!("pack:{pack_id}"), pack_root)
        }
        None => ("project".to_owned(), project_root.join(".base/canon")),
    };

    let relative = scaffold_relative(kind, id);
    let destination = canon_root.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
    if destination.exists() {
        bail!(
            "{} already exists; choose another id",
            destination.display()
        );
    }

    fs::create_dir_all(destination.parent().expect("scaffold path has a parent"))
        .with_context(|| format!("cannot create {}", destination.parent().unwrap().display()))?;
    fs::write(&destination, scaffold_content(kind, id))
        .with_context(|| format!("cannot write {}", destination.display()))?;

    let mut next = vec![format!(
        "edit {} to describe the real behavior",
        display_relative(project_root, &destination)
    )];
    if let Some(pack_id) = &pack {
        next.push(format!(
            "validate the drafted pack with `base pack check {}`",
            canon_root.display()
        ));
        next.push(format!(
            "install it with `base adopt {pack_id}` after it validates"
        ));
    } else {
        next.push("compile it with `base check && base sync`".to_owned());
    }

    let report = NewReport {
        target: root_label,
        kind: kind.name().to_owned(),
        id: id.to_owned(),
        path: display_relative(project_root, &destination),
        next,
    };
    if json {
        print_json(&report)
    } else {
        println!("created {} `{}` at {}", report.kind, report.id, report.path);
        for hint in &report.next {
            println!("  - {hint}");
        }
        Ok(())
    }
}

fn list(project_root: &Path, kind: Option<CanonKind>, json: bool) -> Result<()> {
    let (_config, canon) = super::load_project(project_root)?;
    let mut definitions = Vec::new();
    let wanted = |target: CanonKind| kind.is_none_or(|selected| selected == target);
    if wanted(CanonKind::Rule) {
        for (id, rule) in &canon.rules {
            definitions.push(row("rule", id, &rule.source));
        }
    }
    if wanted(CanonKind::Agent) {
        for (id, agent) in &canon.agents {
            definitions.push(row("agent", id, &agent.source));
        }
    }
    if wanted(CanonKind::Skill) {
        for (id, skill) in &canon.skills {
            definitions.push(row("skill", id, &skill.source));
        }
    }
    if wanted(CanonKind::Stage) {
        for (id, stage) in &canon.stages {
            definitions.push(row("stage", id, &stage.source));
        }
    }
    if wanted(CanonKind::Pipeline) {
        for (id, pipeline) in &canon.pipelines {
            definitions.push(row("pipeline", id, &pipeline.source));
        }
    }
    if wanted(CanonKind::Policy) {
        for (id, policy) in &canon.policies {
            definitions.push(row("policy", id, &policy.source));
        }
    }
    if wanted(CanonKind::Verifier) {
        for (id, verifier) in &canon.verifiers {
            definitions.push(row("verifier", id, &verifier.source));
        }
    }
    if wanted(CanonKind::Knowledge) {
        for (path, knowledge) in &canon.knowledge {
            definitions.push(row("knowledge", path, &knowledge.source));
        }
    }
    definitions.sort_by(|left, right| {
        left.kind
            .cmp(right.kind)
            .then_with(|| left.id.cmp(&right.id))
    });

    let report = ListReport {
        count: definitions.len(),
        definitions,
    };
    if json {
        return print_json(&report);
    }
    if report.definitions.is_empty() {
        println!("no composed canon definitions");
        return Ok(());
    }
    println!("{:<10} {:<28} SOURCE", "KIND", "ID");
    for definition in &report.definitions {
        println!(
            "{:<10} {:<28} {}",
            definition.kind, definition.id, definition.source
        );
    }
    println!();
    println!("{} definitions compose in this project", report.count);
    Ok(())
}

fn row(kind: &'static str, id: &str, source: &crate::canon::Source) -> DefinitionRow {
    DefinitionRow {
        kind,
        id: id.to_owned(),
        source: check::source_label(source),
    }
}

fn scaffold_relative(kind: CanonKind, id: &str) -> String {
    match kind {
        CanonKind::Rule => format!("rules/{id}.md"),
        CanonKind::Agent => format!("agents/{id}.md"),
        CanonKind::Skill => format!("skills/{id}/SKILL.md"),
        CanonKind::Stage => format!("pipelines/stages/{id}.md"),
        CanonKind::Pipeline => format!("pipelines/{id}.md"),
        CanonKind::Policy => format!("policies/{id}.md"),
        CanonKind::Verifier => format!("verifiers/{id}.md"),
        CanonKind::Knowledge => format!("knowledge/{id}.md"),
    }
}

fn scaffold_content(kind: CanonKind, id: &str) -> String {
    match kind {
        CanonKind::Rule => format!(
            "---\nid: {id}\ndescription: Describe when this rule applies.\n---\n\n- State the rule so any agent can follow it without session context.\n"
        ),
        CanonKind::Agent => format!(
            "---\nid: {id}\ndescription: State what this role is responsible for.\ntools:\n  - Read\n  - Grep\n---\n\nDescribe how this role works and what it must verify before finishing.\n"
        ),
        CanonKind::Skill => format!(
            "---\nid: {id}\ndescription: State when to use this skill and what it produces.\n---\n\nWalk through the procedure this skill covers, step by step, so an\nagent can execute it from this file alone.\n"
        ),
        CanonKind::Stage => format!(
            "---\nid: {id}\ndescription: State the artifact this stage owes the run folder.\n---\n\nDescribe the inputs, actions, and expected outputs of this stage.\n"
        ),
        CanonKind::Pipeline => format!(
            "---\nid: {id}\ndescription: State the outcome this workflow produces.\nstages:\n  - use: intake\n  - use: plan\n    gate: plan-approval\n  - use: execute\n  - use: record\n---\n\nDescribe when to invoke this pipeline and how stages hand off. Stages\nresolve by canonical id; every pipeline must end with `record`.\n"
        ),
        CanonKind::Policy => format!(
            "---\nid: {id}\ndescription: State what this lifecycle policy enforces or observes.\nevent: pre-tool-use\nmode: observe\ncommand:\n  - echo\n  - replace-with-your-policy-script\ntimeout-seconds: 10\n---\n\nExplain the command contract: the harness invokes it on the configured\nevent with a JSON payload on stdin.\n"
        ),
        CanonKind::Verifier => format!(
            "---\nid: {id}\ndescription: State what this suite proves when it passes.\nchecks:\n  - id: example\n    run:\n      - echo\n      - replace-with-your-check\n    timeout-seconds: 300\n---\n\nDescribe each check and what its fail and inconclusive outcomes mean.\n"
        ),
        CanonKind::Knowledge => format!(
            "# {id}\n\nRecord the durable lesson: the situation, the action that works, and why.\nLink related entries from knowledge/INDEX.md.\n"
        ),
    }
}

fn display_relative(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.display().to_string())
}

impl CanonKind {
    fn name(self) -> &'static str {
        match self {
            CanonKind::Rule => "rule",
            CanonKind::Agent => "agent",
            CanonKind::Skill => "skill",
            CanonKind::Stage => "stage",
            CanonKind::Pipeline => "pipeline",
            CanonKind::Policy => "policy",
            CanonKind::Verifier => "verifier",
            CanonKind::Knowledge => "knowledge",
        }
    }
}
