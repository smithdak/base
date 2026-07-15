# Canon format

Canon is loaded from `~/.base/canon/` (or `$BASE_HOME/canon/`) and then from
`<repo>/.base/canon/`. Documents are merged by canonical `id`; the project document wins when both
layers define the same ID.

## Documents

Rules, agents, stages, and pipelines are Markdown with YAML frontmatter. IDs must start with a
lowercase letter and contain only lowercase letters, digits, and hyphens.

```yaml
---
id: reviewer
description: Reviews changes for concrete correctness and regression risks.
tools:
  - Read
  - Grep
---

Inspect the diff and relevant surrounding code...
```

Fields by kind:

| Kind | Required fields | Optional fields |
|---|---|---|
| rule | `id` | `description` |
| agent | `id`, `description` | `tools` |
| stage | `id` | `description` |
| pipeline | `id`, `description`, `stages` | authored Markdown body |

A pipeline stage reference uses `use` and may attach one declared stage-approval gate:

```yaml
---
id: build
description: Plan, approve, implement, verify, and record a software change.
stages:
  - use: intake
  - use: plan
    gate: plan-approval
  - use: execute
  - use: record
---
```

Every pipeline must contain at least one stage and end in `record`. Referenced stages and gates must
exist. `base check` validates these invariants before any adapter renders.

Knowledge files are ordinary Markdown under `canon/knowledge/`; `INDEX.md` is the routing entrypoint.
They do not require frontmatter.

## Configuration

Project configuration lives at `.base/base.toml`:

```toml
version = 1
targets = ["claude", "codex", "copilot"]
default_branch = "main"

[[gates]]
id = "plan-approval"
kind = "stage-approval"
description = "Do not execute until the user explicitly approves the written plan."
```

Supported gate kinds are `stage-approval` and `standing-denial`. The `[generated]` table is owned by
`base sync`; it maps project-relative output paths to SHA-256 hashes.

The v1 CLI has a mechanical adapter binding for the built-in `never-push-default-branch` standing
denial. Additional standing-denial IDs still compile into instructions, but `base check` reports
them as advisory until an explicit adapter binding exists.
