# Plan: fix pipeline and verify stage

## Files to create

**`.base/canon/pipelines/stages/diagnose.md`** — reproduce before planning:

> Reproduce the reported behavior first and capture the failing command and its output in the run
> folder as `diagnosis.md`, then isolate the smallest cause that explains it. State the root cause,
> the affected surface, and nearby risks. If the failure cannot be reproduced, stop and report that
> instead of guessing at a fix.

**`.base/canon/pipelines/stages/verify.md`** — evidence-backed completion:

> Run the acceptance checks named in `task.md` and the approved plan. Capture the commands and
> their real output as files under `evidence/`; reference them from `result.md` with a pass or
> fail verdict per check. A failing check means the run is not `completed` — fix it or record the
> honest outcome. Never substitute a claim for a captured result.

**`.base/canon/pipelines/fix.md`** — the pipeline:

```yaml
id: fix
description: Diagnose, plan, approve, repair, and prove a defect fix.
stages:
  - use: intake
  - use: diagnose
  - use: plan
    gate: plan-approval
  - use: execute
  - use: verify
  - use: record
```

Body: use when something is broken and the cause is not yet established; the diagnosis bounds the
plan — do not plan past what the reproduction demonstrates.

## Files to change

- **`.base/canon/pipelines/build.md`** — insert `- use: verify` between `execute` and `record`.
  The description already promises verification; this makes the stage sequence deliver it.

## Verification commands

1. `base check` → canon valid, 6 stages, 2 pipelines.
2. `base sync` → new fix surfaces for claude/codex/copilot; then `base sync --check` → clean.
3. Fresh clone of the feature branch → `base sync --check` → clean.
4. Grep compiled `.claude/skills/build/SKILL.md` for the Verify section ordering.
5. Capture 1–4 under `evidence/checks.md`.

## Delivery

Feature branch `feat/fix-pipeline-verify-stage`; commit canon sources + regenerated surfaces +
run artifacts + history line together; push branch; open PR to `main` via `gh`.

## Risks

- The verify stage makes `build` runs slightly heavier (every run must capture evidence). This is
  the deliberate direction of D-008; accepting it here is the point.
- Global canon (`~/.base`) now duplicates the project scaffold IDs. Project wins on merge, so no
  behavioral change in this repo, but the global copy is not yet version-controlled — noted as a
  possible follow-up (`git init ~/.base`), out of scope here.
