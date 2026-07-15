use std::env;
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::canon::Layer;
use crate::config::Target;
use crate::render::enforcement;

use super::{load_project, print_json};

#[derive(Debug, Serialize)]
struct CheckReport {
    valid: bool,
    rules: usize,
    agents: usize,
    stages: usize,
    pipelines: usize,
    knowledge: usize,
    enforcement: Vec<EnforcementRow>,
    warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
struct EnforcementRow {
    gate: String,
    kind: String,
    target: String,
    fidelity: &'static str,
}

pub fn run(project_root: &Path, json: bool) -> Result<()> {
    let (config, canon) = load_project(project_root)?;
    let hook_binary_resolves = hook_binary_on_path();
    let mut rows = Vec::new();
    let mut warnings = Vec::new();
    for gate in &config.gates {
        for target in &config.targets {
            let mut fidelity = enforcement(gate, *target);
            // Claude enforcement is a PreToolUse hook that runs `base` from PATH;
            // command-not-found hook errors are non-blocking, so without the binary
            // only the deny rules remain and implicit pushes pass through.
            if fidelity == "enforced" && *target == Target::Claude && !hook_binary_resolves {
                fidelity = "assisted";
                warnings.push(format!(
                    "`base` does not resolve on PATH, so the Claude PreToolUse hook for `{}` cannot run; only deny rules remain and implicit pushes are not blocked. Install base on PATH (e.g. `cargo install --path .`) to restore enforcement.",
                    gate.id
                ));
            }
            rows.push(EnforcementRow {
                gate: gate.id.clone(),
                kind: gate.kind.to_string(),
                target: target.to_string(),
                fidelity,
            });
        }
    }
    let mut warn_global_only = |kind: &str, id: &str, adopt_path: &str, usable: &str| {
        warnings.push(format!(
            "global-only {kind} `{id}` {usable}; copy it into `{adopt_path}` to adopt it in this project"
        ));
    };
    for (id, rule) in &canon.rules {
        if rule.source.layer == Layer::Global {
            warn_global_only(
                "rule",
                id,
                ".base/canon/rules/",
                "is not rendered into committed surfaces",
            );
        }
    }
    for (id, agent) in &canon.agents {
        if agent.source.layer == Layer::Global {
            warn_global_only(
                "agent",
                id,
                ".base/canon/agents/",
                "is not rendered into committed surfaces",
            );
        }
    }
    for (id, stage) in &canon.stages {
        if stage.source.layer == Layer::Global {
            warn_global_only(
                "stage",
                id,
                ".base/canon/pipelines/stages/",
                "is not usable by project pipelines",
            );
        }
    }
    for (id, pipeline) in &canon.pipelines {
        if pipeline.source.layer == Layer::Global {
            warn_global_only(
                "pipeline",
                id,
                ".base/canon/pipelines/",
                "is not rendered into committed surfaces",
            );
        }
    }
    for (path, entry) in &canon.knowledge {
        if entry.source.layer == Layer::Global {
            warn_global_only(
                "knowledge",
                path,
                ".base/canon/knowledge/",
                "is not rendered into committed surfaces",
            );
        }
    }
    let report = CheckReport {
        valid: true,
        rules: canon.rules.len(),
        agents: canon.agents.len(),
        stages: canon.stages.len(),
        pipelines: canon.pipelines.len(),
        knowledge: canon.knowledge.len(),
        enforcement: rows,
        warnings,
    };
    if json {
        return print_json(&report);
    }

    println!(
        "canon valid: {} rules, {} agents, {} stages, {} pipelines, {} knowledge entries",
        report.rules, report.agents, report.stages, report.pipelines, report.knowledge
    );
    println!();
    println!("{:<29} {:<17} {:<9} FIDELITY", "GATE", "KIND", "TARGET");
    for row in &report.enforcement {
        println!(
            "{:<29} {:<17} {:<9} {}",
            row.gate, row.kind, row.target, row.fidelity
        );
    }
    for warning in &report.warnings {
        println!();
        println!("warning: {warning}");
    }
    print_notes(&config);
    Ok(())
}

fn hook_binary_on_path() -> bool {
    let Some(path) = env::var_os("PATH") else {
        return false;
    };
    let name = if cfg!(windows) { "base.exe" } else { "base" };
    env::split_paths(&path).any(|dir| !dir.as_os_str().is_empty() && is_executable(&dir.join(name)))
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path)
        .map(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

fn print_notes(config: &crate::config::Config) {
    println!();
    if config.targets.contains(&Target::Claude) {
        println!(
            "claude: standing denial uses a project permission deny plus PreToolUse hooks over Bash git pushes and GitHub MCP branch writes (PR merges stay the review path); stage approval is prompt-assisted"
        );
    }
    if config.targets.contains(&Target::Codex) {
        println!(
            "codex: explicit default-branch refspecs are blocked by project rules; stage approval and unusual refspecs remain assisted"
        );
    }
    if config.targets.contains(&Target::Copilot) {
        println!("copilot: gates are declared in prose and remain advisory");
    }
}
