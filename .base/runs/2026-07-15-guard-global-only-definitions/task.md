# Task: guard global-only rules, agents, and pipelines in committed output (W-0005)

## Outcome

The D-017 principle — nothing outside the repo is ever the sole source of committed bytes — holds
for every canon kind. A global-only rule, agent, stage, or pipeline never enters generated
surfaces; every exclusion is reported by `base check`; adoption is a copy into the project canon.

## Constraints

- Same posture as D-017: exclude honestly (check warnings with the adoption path), never silently.
- Watch the composition vector: stages are inlined into pipeline skills, so a *project* pipeline
  referencing a *global-only* stage would smuggle global bytes into committed output even with
  per-kind render filters. That must be a hard validation error, not a warning.
- This repo's own surfaces must not change (all definitions are project-resident here).

## Assumptions

- Merged-map semantics from W-0004 carry over: project entries shadow same-ID global entries, so
  any Layer::Global entry remaining in a merged map is global-only by construction.

## Acceptance checks (from W-0005)

1. A decision (D-018) extends the D-017 treatment to rules, agents, stages, and pipelines,
   including the cross-layer stage-reference rule.
2. Reproduced failing first, then green: a clone synced against a global canon holding a
   global-only rule, agent, and pipeline passes `sync --check` on a machine without that global
   canon.
3. `base check` reports every excluded global-only definition by kind and id, with its adoption
   path; a project pipeline referencing a global-only stage fails validation with a copy-to-adopt
   message.
4. `cargo fmt` / `clippy -D warnings` / full suite / `base sync` (0 written here) /
   `sync --check` clean.
