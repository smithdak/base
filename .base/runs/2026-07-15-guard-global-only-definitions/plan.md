# Plan: extend D-017 to all canon kinds (W-0005)

The design choice was made in D-017 (library + adopt-by-copy) and W-0005's first criterion names
it; this plan applies it. D-018 records the extension and the new validation rule.

## Changes

**`src/render.rs`** — layer-filter every render surface to `Layer::Project`:
- `render_instructions`: rules, agents, pipelines sections
- `render_claude`: the per-agent file loop and per-pipeline skill loop
- `render_codex` / `render_copilot`: the per-pipeline loops

**`src/canon.rs`** — `validate` gains the cross-layer composition rule: a **project** pipeline may
reference only **project** stages; violation is an error naming the fix ("copy the stage into
`.base/canon/pipelines/stages/` to adopt it"). Global pipelines are exempt (they never render).
This keeps `render_pipeline`'s stage lookup safe by construction.

**`src/commands/check.rs`** — generalize the W-0004 warning: one warning per global-only rule,
agent, stage, and pipeline, each naming its kind, id, and adoption path (`.base/canon/rules/`,
`canon/agents/`, `canon/pipelines/stages/`, `canon/pipelines/`). Stage wording: "not usable by
project pipelines until copied", since stages never render directly.

**`docs/DECISIONS.md`** — D-018: extends D-017 to all kinds; amends D-007's compile model in
effect (the global layer seeds and serves as a library; only repo-resident definitions compile
into committed surfaces); records the stage-reference validation rule.

**`docs/SPEC.md` §2** — one sentence after the two-layer flow paragraph stating the rule
("Only repo-resident definitions enter committed surfaces; the global layer seeds new projects
and is adopted by copy — D-017/D-018."). §7's tethered table is untouched.

**`tests/cli.rs`** — extend the global-only test (or add one): global-only rule + agent + stage +
pipeline → sync output byte-identical to baseline, no generated files for them, `check --json`
warnings list all four; a project pipeline referencing a global-only stage → `check`/`sync` fail
with the adopt message.

## Verification

1. Re-run the diagnosis scenario: step-3 `sync --check` without the global canon must exit 0, and
   step 1 must write 0 files.
2. `cargo fmt` / `clippy -D warnings` / full suite; this repo: `base sync` writes 0, `sync --check`
   green (all definitions here are project-resident).
3. Rebuild + reinstall release binary; before/after evidence in the run folder.

## Risks

- This narrows D-007's original "compiles global + overlay" promise to seed-and-adopt semantics —
  deliberate, and D-018 says so out loud rather than leaving the model ambiguous.
- A repo whose only pipelines were global-canon would now render no skills; `validate` still
  requires ≥1 pipeline in the merged canon, and `check` warnings make the situation legible.
  Accepted for v1's single-user reality.

## Delivery

Branch `fix/w-0005-global-only-definitions` from `main` (carries the pending W-0004 verdict state
file, noted in the PR); run artifacts + history line; PR to `main`.
