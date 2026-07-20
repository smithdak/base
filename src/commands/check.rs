use std::env;
use std::path::Path;
use std::process::Command;

use anyhow::Result;
use serde::Serialize;

use crate::canon::Layer;
use crate::config::Target;
use crate::render::{agent_path, enforcement, policy_fidelity, skill_path};

use super::{load_project, print_json};

#[derive(Debug, Serialize)]
struct CheckReport {
    valid: bool,
    packs: Vec<PackSummary>,
    rules: usize,
    agents: usize,
    skills: usize,
    stages: usize,
    pipelines: usize,
    policies: usize,
    verifiers: usize,
    knowledge: usize,
    overrides: Vec<OverrideRow>,
    enforcement: Vec<EnforcementRow>,
    surfaces: Vec<SurfaceRow>,
    warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
struct PackSummary {
    id: String,
    version: String,
    files: usize,
}

#[derive(Debug, Serialize)]
struct EnforcementRow {
    gate: String,
    kind: String,
    target: String,
    fidelity: &'static str,
    profile: &'static str,
    scope: &'static str,
    prerequisite: &'static str,
}

#[derive(Debug, Serialize)]
struct SurfaceRow {
    kind: &'static str,
    id: String,
    target: String,
    fidelity: &'static str,
    profile: &'static str,
    scope: &'static str,
    output: String,
}

#[derive(Debug, Serialize)]
struct OverrideRow {
    kind: &'static str,
    id: String,
    replaced: String,
    winner: String,
}

pub fn run(project_root: &Path, json: bool) -> Result<()> {
    let (config, canon) = load_project(project_root)?;
    super::work::validate(project_root)?;
    let hook_runtime = hook_runtime_on_path(config.requires_base.as_deref());
    let mut rows = Vec::new();
    let mut surfaces = Vec::new();
    let mut warnings = Vec::new();
    if config.requires_base.is_none() {
        warnings.push(
            "project does not declare `requires-base`; runtime compatibility is not reproducible across the team"
                .to_owned(),
        );
    }
    for gate in &config.gates {
        for target in &config.targets {
            let mut fidelity = enforcement(gate, *target);
            if matches!(fidelity, "native-hook" | "hybrid-hook") && !hook_runtime.ready {
                fidelity = "assisted";
                warnings.push(format!(
                    "Base hook runtime is unavailable on PATH ({}), so the {} lifecycle hook for `{}` cannot run; install a compatible Base in the agent environment to restore native-hook fidelity.",
                    hook_runtime.reason, target, gate.id
                ));
            }
            let (profile, scope, prerequisite) = hook_profile(*target);
            rows.push(EnforcementRow {
                gate: gate.id.clone(),
                kind: gate.kind.to_string(),
                target: target.to_string(),
                fidelity,
                profile,
                scope,
                prerequisite,
            });
        }
    }

    for agent in canon
        .agents
        .values()
        .filter(|item| item.source.repo_resident())
    {
        for target in &config.targets {
            surfaces.push(SurfaceRow {
                kind: "agent",
                id: agent.meta.id.clone(),
                target: target.to_string(),
                fidelity: "native",
                profile: agent_profile(*target),
                scope: agent_scope(*target),
                output: agent_path(&agent.meta.id, *target),
            });
        }
    }
    for skill in canon
        .skills
        .values()
        .filter(|item| item.source.repo_resident())
    {
        for target in &config.targets {
            surfaces.push(SurfaceRow {
                kind: "skill",
                id: skill.meta.id.clone(),
                target: target.to_string(),
                fidelity: "native",
                profile: skill_profile(*target),
                scope: skill_scope(*target),
                output: skill_path(&skill.meta.id, *target),
            });
        }
    }
    for pipeline in canon
        .pipelines
        .values()
        .filter(|item| item.source.repo_resident())
    {
        for target in &config.targets {
            match target {
                Target::Claude => surfaces.push(SurfaceRow {
                    kind: "pipeline",
                    id: pipeline.meta.id.clone(),
                    target: target.to_string(),
                    fidelity: "native",
                    profile: "claude-skill",
                    scope: "claude-code",
                    output: format!(".claude/skills/{}/SKILL.md", pipeline.meta.id),
                }),
                Target::Codex => surfaces.push(SurfaceRow {
                    kind: "pipeline",
                    id: pipeline.meta.id.clone(),
                    target: target.to_string(),
                    fidelity: "native",
                    profile: "agent-skill",
                    scope: "codex",
                    output: format!(".agents/skills/{}/SKILL.md", pipeline.meta.id),
                }),
                Target::Copilot => {
                    surfaces.push(SurfaceRow {
                        kind: "pipeline",
                        id: pipeline.meta.id.clone(),
                        target: target.to_string(),
                        fidelity: "native",
                        profile: "agent-skill",
                        scope: "copilot-cli-cloud",
                        output: format!(".agents/skills/{}/SKILL.md", pipeline.meta.id),
                    });
                    surfaces.push(SurfaceRow {
                        kind: "pipeline",
                        id: pipeline.meta.id.clone(),
                        target: target.to_string(),
                        fidelity: "native",
                        profile: "vscode-prompt-file",
                        scope: "vscode-only",
                        output: format!(".github/prompts/{}.prompt.md", pipeline.meta.id),
                    });
                }
            }
        }
    }
    for policy in canon
        .policies
        .values()
        .filter(|item| item.source.repo_resident())
    {
        for target in &config.targets {
            let mut fidelity = policy_fidelity(policy, *target);
            if matches!(fidelity, "native-hook" | "partial-hook") && !hook_runtime.ready {
                fidelity = "assisted";
            }
            if policy_fidelity(policy, *target) == "partial-hook" {
                let reason = match target {
                    Target::Codex => {
                        "Codex does not invoke hooks for every hosted tool; use Bash, apply_patch, or mcp__ matchers"
                    }
                    Target::Claude | Target::Copilot => {
                        "one or more match-tools entries are outside Base's verified portable alias set"
                    }
                };
                warnings.push(format!(
                    "policy `{}` has partial {} tool-event coverage: {reason}",
                    policy.meta.id, target
                ));
            }
            let (profile, scope, _) = hook_profile(*target);
            surfaces.push(SurfaceRow {
                kind: "policy",
                id: policy.meta.id.clone(),
                target: target.to_string(),
                fidelity,
                profile,
                scope,
                output: match target {
                    Target::Claude => ".claude/settings.json".to_owned(),
                    Target::Codex if fidelity != "assisted" => ".codex/hooks.json".to_owned(),
                    Target::Codex => "AGENTS.md (assisted contract)".to_owned(),
                    Target::Copilot => ".github/hooks/base.json".to_owned(),
                },
            });
        }
    }
    for suite in canon
        .verifiers
        .values()
        .filter(|item| item.source.repo_resident())
    {
        for target in &config.targets {
            surfaces.push(SurfaceRow {
                kind: "verifier",
                id: suite.meta.id.clone(),
                target: target.to_string(),
                fidelity: "assisted",
                profile: "base-cli",
                scope: "repository",
                output: format!("base verify {}", suite.meta.id),
            });
        }
    }

    let mut warn_global_only = |kind: &str, id: &str, adopt_path: &str, usable: &str| {
        warnings.push(format!(
            "global-only {kind} `{id}` {usable}; copy it into `{adopt_path}` or install its versioned pack to adopt it in this project"
        ));
    };
    for (id, rule) in &canon.rules {
        if rule.source.layer == Layer::Global {
            warn_global_only("rule", id, ".base/canon/rules/", "is not rendered");
        }
    }
    for (id, agent) in &canon.agents {
        if agent.source.layer == Layer::Global {
            warn_global_only("agent", id, ".base/canon/agents/", "is not rendered");
        }
    }
    for (id, skill) in &canon.skills {
        if skill.source.layer == Layer::Global {
            warn_global_only("skill", id, ".base/canon/skills/", "is not rendered");
        }
    }
    for (id, stage) in &canon.stages {
        if stage.source.layer == Layer::Global {
            warn_global_only(
                "stage",
                id,
                ".base/canon/pipelines/stages/",
                "is not usable by repository pipelines",
            );
        }
    }
    for (id, pipeline) in &canon.pipelines {
        if pipeline.source.layer == Layer::Global {
            warn_global_only("pipeline", id, ".base/canon/pipelines/", "is not rendered");
        }
    }
    for (id, policy) in &canon.policies {
        if policy.source.layer == Layer::Global {
            warn_global_only("policy", id, ".base/canon/policies/", "is not bound");
        }
    }
    for (id, verifier) in &canon.verifiers {
        if verifier.source.layer == Layer::Global {
            warn_global_only(
                "verifier",
                id,
                ".base/canon/verifiers/",
                "cannot be executed by this project",
            );
        }
    }
    for (path, entry) in &canon.knowledge {
        if entry.source.layer == Layer::Global {
            warn_global_only(
                "knowledge",
                path,
                ".base/canon/knowledge/",
                "is not rendered",
            );
        }
    }

    let report = CheckReport {
        valid: true,
        packs: config
            .packs
            .iter()
            .map(|pack| PackSummary {
                id: pack.id.clone(),
                version: pack.version.clone(),
                files: pack.files.len(),
            })
            .collect(),
        rules: canon.rules.len(),
        agents: canon.agents.len(),
        skills: canon.skills.len(),
        stages: canon.stages.len(),
        pipelines: canon.pipelines.len(),
        policies: canon.policies.len(),
        verifiers: canon.verifiers.len(),
        knowledge: canon.knowledge.len(),
        overrides: canon
            .overrides
            .iter()
            .map(|item| OverrideRow {
                kind: item.kind,
                id: item.id.clone(),
                replaced: source_label(&item.replaced),
                winner: source_label(&item.winner),
            })
            .collect(),
        enforcement: rows,
        surfaces,
        warnings,
    };
    if json {
        return print_json(&report);
    }

    println!(
        "canon valid: {} packs, {} rules, {} agents, {} skills, {} stages, {} pipelines, {} policies, {} verifiers, {} knowledge entries",
        report.packs.len(),
        report.rules,
        report.agents,
        report.skills,
        report.stages,
        report.pipelines,
        report.policies,
        report.verifiers,
        report.knowledge
    );
    if !report.overrides.is_empty() {
        println!();
        println!("{:<10} {:<24} {:<20} WINNER", "KIND", "ID", "REPLACED");
        for row in &report.overrides {
            println!(
                "{:<10} {:<24} {:<20} {}",
                row.kind, row.id, row.replaced, row.winner
            );
        }
    }
    println!();
    println!(
        "{:<29} {:<17} {:<9} {:<12} {:<20} SCOPE",
        "GATE", "KIND", "TARGET", "FIDELITY", "PROFILE"
    );
    for row in &report.enforcement {
        println!(
            "{:<29} {:<17} {:<9} {:<12} {:<20} {}",
            row.gate, row.kind, row.target, row.fidelity, row.profile, row.scope
        );
        println!("  prerequisite: {}", row.prerequisite);
    }
    if !report.surfaces.is_empty() {
        println!();
        println!(
            "{:<10} {:<24} {:<9} {:<12} {:<20} {:<20} OUTPUT",
            "KIND", "ID", "TARGET", "FIDELITY", "PROFILE", "SCOPE"
        );
        for row in &report.surfaces {
            println!(
                "{:<10} {:<24} {:<9} {:<12} {:<20} {:<20} {}",
                row.kind, row.id, row.target, row.fidelity, row.profile, row.scope, row.output
            );
        }
    }
    for warning in &report.warnings {
        println!();
        println!("warning: {warning}");
    }
    print_notes(&config);
    Ok(())
}

fn source_label(source: &crate::canon::Source) -> String {
    match source.layer {
        Layer::Global => "global".to_owned(),
        Layer::Pack => format!("pack:{}", source.pack.as_deref().unwrap_or("unknown")),
        Layer::Project => "project".to_owned(),
    }
}

fn hook_profile(target: Target) -> (&'static str, &'static str, &'static str) {
    match target {
        Target::Claude => (
            "claude-project-hook",
            "claude-code",
            "compatible Base on PATH and project hooks enabled",
        ),
        Target::Codex => (
            "codex-project-hook",
            "codex-local",
            "compatible Base on PATH, hooks feature enabled, and exact .codex hook definition trusted",
        ),
        Target::Copilot => (
            "copilot-repo-hook",
            "copilot-cli-cloud",
            "compatible Base on PATH and repository hooks enabled; host timeouts are fail-open",
        ),
    }
}

fn agent_profile(target: Target) -> &'static str {
    match target {
        Target::Claude => "claude-subagent",
        Target::Codex => "codex-subagent",
        Target::Copilot => "copilot-custom-agent",
    }
}

fn agent_scope(target: Target) -> &'static str {
    match target {
        Target::Claude => "claude-code",
        Target::Codex => "codex",
        Target::Copilot => "copilot-ide-cli-cloud",
    }
}

fn skill_profile(target: Target) -> &'static str {
    match target {
        Target::Claude => "claude-skill",
        Target::Codex | Target::Copilot => "agent-skill",
    }
}

fn skill_scope(target: Target) -> &'static str {
    match target {
        Target::Claude => "claude-code",
        Target::Codex => "codex",
        Target::Copilot => "copilot-cli-cloud",
    }
}

struct HookRuntime {
    ready: bool,
    reason: String,
}

fn hook_runtime_on_path(requirement: Option<&str>) -> HookRuntime {
    let Some(path) = env::var_os("PATH") else {
        return HookRuntime {
            ready: false,
            reason: "PATH is unset".to_owned(),
        };
    };
    let name = if cfg!(windows) { "base.exe" } else { "base" };
    let Some(binary) = env::split_paths(&path)
        .filter(|dir| !dir.as_os_str().is_empty())
        .map(|dir| dir.join(name))
        .find(|path| is_executable(path))
    else {
        return HookRuntime {
            ready: false,
            reason: "no base executable resolved".to_owned(),
        };
    };
    let mut command = Command::new(&binary);
    command.args([
        "__hook",
        "capabilities",
        "--require-feature",
        "pre-tool",
        "--require-feature",
        "policy",
    ]);
    if let Some(requirement) = requirement {
        command.args(["--require", requirement]);
    }
    let output = command.output();
    match output {
        Ok(output) if output.status.success() => {
            let value = serde_json::from_slice::<serde_json::Value>(&output.stdout).ok();
            let compatible = value.as_ref().is_some_and(|value| {
                let features = value.get("features").and_then(serde_json::Value::as_array);
                value.get("protocol").and_then(serde_json::Value::as_u64) == Some(1)
                    && value
                        .get("version")
                        .and_then(serde_json::Value::as_str)
                        .and_then(|version| semver::Version::parse(version).ok())
                        .is_some()
                    && ["pre-tool", "policy"].iter().all(|required| {
                        features.is_some_and(|features| {
                            features
                                .iter()
                                .any(|feature| feature.as_str() == Some(required))
                        })
                    })
            });
            if compatible {
                HookRuntime {
                    ready: true,
                    reason: format!("{} supports hook protocol 1", binary.display()),
                }
            } else {
                HookRuntime {
                    ready: false,
                    reason: format!(
                        "{} returned an incompatible capability record",
                        binary.display()
                    ),
                }
            }
        }
        Ok(_) => HookRuntime {
            ready: false,
            reason: format!("{} predates hook protocol 1", binary.display()),
        },
        Err(error) => HookRuntime {
            ready: false,
            reason: format!("{} could not be probed: {error}", binary.display()),
        },
    }
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
    let artifact_gates = config.gates.iter().any(|gate| {
        gate.kind == crate::config::GateKind::StageApproval && gate.satisfied_by.is_some()
    });
    println!();
    if config.targets.contains(&Target::Claude) {
        let stage_note = if artifact_gates {
            "stage approval is enforced through approval artifacts"
        } else {
            "stage approval is prompt-assisted"
        };
        println!(
            "claude: native agents, skills, and lifecycle hooks; standing denial uses permission rules plus PreToolUse; {stage_note}"
        );
    }
    if config.targets.contains(&Target::Codex) {
        println!(
            "codex: native agents, Agent Skills, and repository hooks; project hooks require compatible Base plus explicit trust through `/hooks`; project rules remain defense in depth"
        );
    }
    if config.targets.contains(&Target::Copilot) {
        println!(
            "copilot: custom agents, Agent Skills for CLI/cloud, VS Code prompt files, and CLI/cloud repository hooks; hooks require Base in the execution environment and host timeouts remain fail-open"
        );
    }
}
