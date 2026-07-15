use std::collections::BTreeMap;

use crate::config::Config;

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

Compile reusable repository workflows to skills for Claude Code (`.claude/skills/`) and Codex
(`.agents/skills/`). Codex custom prompts are deprecated and user-scoped. GitHub Copilot repository
prompt files remain useful but are IDE-dependent and public preview.

Report gate fidelity per gate and target. A declared policy is not mechanically enforced until the
adapter emits and verifies a native binding for that exact policy.
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
    files.insert(".base/work/.gitkeep".to_owned(), String::new());
    files.insert(".base/runs/.gitkeep".to_owned(), String::new());
    files.insert(".base/knowledge/.gitkeep".to_owned(), String::new());
    files
}

fn path(prefix: &str, suffix: &str) -> String {
    format!("{prefix}{suffix}")
}
