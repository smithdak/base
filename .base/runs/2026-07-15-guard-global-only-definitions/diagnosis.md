# Diagnosis: global-only definitions leak whole payloads into committed output (W-0005)

## Reproduction (evidence/repro-before.md)

A scratch global canon with one global-only rule, agent, stage, and pipeline; one `sync` in a
clean clone wrote **7 files**: the rule's full body into all three instruction files, an agent
bullet and `/ship` pipeline entry alongside it, and four brand-new generated files
(`.claude/agents/researcher.md`, `ship` skills for all three targets). On a machine without that
global canon, `sync --check` fails with 3 content drifts + 4 stale manifest entries. Exit 1.

## Root cause — src/render.rs, all render paths

W-0004's fix covered only the knowledge pointer list. Every other section still iterates merged
maps without layer filtering:

- `render_instructions` — rules (full bodies), agents (bullets), pipelines (invocation lines)
- `render_claude` — a generated agent file per agent, a skill per pipeline
- `render_codex` / `render_copilot` — a skill/prompt per pipeline

Same merged-map guarantee as W-0004: project entries shadow same-ID global entries, so every
remaining Layer::Global entry has no repo-resident source by construction.

## The composition vector (found by reading, confirmed by design)

Stages are not rendered standalone — `render_pipeline` **inlines** stage bodies into skill files
(`canon.stages.get(...)`, layer-blind). Per-kind render filters are therefore insufficient: a
*project* pipeline referencing a *global-only* stage would still smuggle global bytes into a
committed skill. This must fail validation (adopt the stage by copy), not merely warn — a warning
would leave environment-dependent bytes in place.

## Blast radius

Any machine where `~/.base/canon` gains a definition this repo lacks silently rewrites committed
surfaces on next sync; CI and collaborators then see unexplained drift. Inverse of W-0004's
severity: not a dangling pointer but full foreign content.
