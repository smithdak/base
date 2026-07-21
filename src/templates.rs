use std::collections::BTreeMap;

use crate::config::Config;

pub fn global_files() -> BTreeMap<String, String> {
    let mut files = canon_files("");
    files.extend(global_pack_files());
    files
}

pub fn global_pack_files() -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    for (relative, content) in software_delivery_pack_files() {
        files.insert(format!("canon/packs/software-delivery/{relative}"), content);
    }
    files
}

pub fn canon_files(prefix: &str) -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    files.insert(
        path(prefix, "canon/rules/working-agreements.md"),
        r#"---
id: working-agreements
description: Durable working rules for every harness.
---

- Read repository guidance and the relevant code before changing anything.
- Keep changes scoped to the approved task and preserve unrelated user work.
- Verify behavior in proportion to risk and report concrete evidence.
- Do not claim completion while required work or failing checks remain.
"#
        .to_owned(),
    );
    files.insert(
        path(prefix, "canon/agents/builder.md"),
        r#"---
id: builder
description: Implements approved plans and verifies the resulting behavior.
tools:
  - Read
  - Edit
  - Bash
---

Implement the smallest cohesive change that satisfies the approved plan. Preserve existing
conventions, add or update focused tests, and leave the workspace in a verifiable state.
"#
        .to_owned(),
    );
    files.insert(
        path(prefix, "canon/agents/reviewer.md"),
        r#"---
id: reviewer
description: Reviews changes for correctness, regressions, safety, and missing verification.
tools:
  - Read
  - Grep
  - Bash
---

Inspect the diff and relevant surrounding code. Prioritize concrete correctness, security,
data-loss, and compatibility risks. Cite exact files and explain how each finding can fail.
"#
        .to_owned(),
    );
    files.insert(
        path(prefix, "canon/pipelines/stages/intake.md"),
        r#"---
id: intake
description: Turn the request into a bounded task artifact.
---

Clarify the requested outcome from available context. Inspect the repository before asking
questions that the files can answer. Create `.base/runs/YYYY-MM-DD-<short-kebab-slug>/` (add a
numeric suffix on collision) and write `task.md` containing the outcome, constraints, assumptions,
and acceptance checks. Reserve an empty `evidence/` folder.
"#
        .to_owned(),
    );
    files.insert(
        path(prefix, "canon/pipelines/stages/plan.md"),
        r#"---
id: plan
description: Produce a concrete implementation plan.
---

Write `plan.md` in the run folder. Name the files or components to change, the intended behavior,
the verification commands, and material risks. Keep the plan executable by another capable agent.
"#
        .to_owned(),
    );
    files.insert(
        path(prefix, "canon/pipelines/stages/execute.md"),
        r#"---
id: execute
description: Implement and verify the approved plan.
---

Implement the approved plan without widening scope. Run the planned checks and capture a concise
`result.md` with changed paths, verification results, and any remaining limitations. Put durable
proof files under `evidence/` when useful.
"#
        .to_owned(),
    );
    files.insert(
        path(prefix, "canon/pipelines/stages/record.md"),
        r#"---
id: record
description: Record every pipeline exit in the project ledger.
---

Always run this stage, including after rejection, failure, or abort. Append exactly one compact JSON
object line to `.base/history.jsonl` with `slug`, `date`, `pipeline`, `harness`, `outcome`, and
`paths`. Use `completed`, `aborted`, or `failed` for `outcome`; never rewrite previous lines.
"#
        .to_owned(),
    );
    files.insert(
        path(prefix, "canon/pipelines/build.md"),
        r#"---
id: build
description: Plan, approve, implement, verify, and record a software change.
stages:
  - use: intake
  - use: plan
    gate: plan-approval
  - use: execute
  - use: record
---

Use this pipeline for repository changes that should leave an auditable plan, result, and history
entry. Treat the user's invocation text as the task; do not invent a separate objective.
"#
        .to_owned(),
    );
    files.insert(
        path(prefix, "canon/knowledge/INDEX.md"),
        r#"# Knowledge index

- [Harness pipeline surfaces](adapter-surfaces.md) — current repo-scoped workflow targets and the
  rule for reporting gate fidelity.

Add short links here when another lesson is promoted into canon. Load only the entries relevant to
the current task.
"#
        .to_owned(),
    );
    files.insert(
        path(prefix, "canon/knowledge/adapter-surfaces.md"),
        r#"# Harness pipeline surfaces

Compile repository skills to Claude Code (`.claude/skills/`) and the open Agent Skills surface
(`.agents/skills/`) shared by Codex and GitHub Copilot. Compile agents to each target's native
project surface. Pipelines use a Claude skill, a shared Codex/Copilot Agent Skill, and a separate VS
Code prompt-file profile for Copilot.

All three targets have repository lifecycle hooks for equivalent events. Codex project hooks
require explicit trust; Codex session-end remains assisted because `Stop` is turn-scoped. Hook
execution requires a `requires-base`-compatible binary in the target environment.

Report fidelity, product profile, scope, and runtime/trust prerequisites per target. `native-hook`
means a documented lifecycle binding, not an authorization boundary; `hybrid-hook` means pending
approval is mechanical while denial routing remains behavioral; `partial-hook` means the native
binding does not cover the complete declared tool domain. Allowlisted target-specific migration
input lives under `.base/native/` and is composed into hash-owned output. Protect default branches
at the Git server. Copilot sanitizes the current built-in GitHub MCP namespace to
`github-mcp-server-*`; map it explicitly, and report arbitrary unmapped Copilot MCP matchers as
`partial-hook` rather than guessing.
"#
        .to_owned(),
    );
    files
}

pub fn project_files() -> BTreeMap<String, String> {
    let mut files = canon_files(".base/");
    files.insert(
        ".base/base.toml".to_owned(),
        toml::to_string_pretty(&Config::default()).expect("default config serializes"),
    );
    files.insert(".base/history.jsonl".to_owned(), String::new());
    files.insert(".base/.gitignore".to_owned(), ".lock\n".to_owned());
    files.insert(".base/work/.gitkeep".to_owned(), String::new());
    files.insert(".base/runs/.gitkeep".to_owned(), String::new());
    files.insert(".base/knowledge/.gitkeep".to_owned(), String::new());
    files.insert(".base/packs/.gitkeep".to_owned(), String::new());
    files.insert(".base/native/.gitkeep".to_owned(), String::new());
    files.insert(".base/state/.gitkeep".to_owned(), String::new());
    files
}

fn software_delivery_pack_files() -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    macro_rules! asset {
        ($path:literal) => {
            files.insert(
                $path.to_owned(),
                include_str!(concat!("../assets/packs/software-delivery/", $path)).to_owned(),
            );
        };
    }
    asset!("pack.md");
    asset!("rules/delivery-discipline.md");
    asset!("agents/delivery-analyst.md");
    asset!("agents/delivery-implementer.md");
    asset!("agents/delivery-auditor.md");
    asset!("skills/pickup/SKILL.md");
    asset!("skills/durable-handoff/SKILL.md");
    asset!("skills/evidence-review/SKILL.md");
    asset!("skills/decision-record/SKILL.md");
    asset!("policies/session-context.md");
    asset!("verifiers/delivery-foundation.md");
    asset!("pipelines/stages/delivery-discover.md");
    asset!("pipelines/stages/delivery-plan.md");
    asset!("pipelines/stages/delivery-implement.md");
    asset!("pipelines/stages/delivery-prove.md");
    asset!("pipelines/stages/delivery-review.md");
    asset!("pipelines/delivery.md");
    asset!("knowledge/operating-model.md");
    files
}

fn path(prefix: &str, suffix: &str) -> String {
    format!("{prefix}{suffix}")
}
