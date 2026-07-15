# Plan: reconcile global knowledge with reproducible output (W-0004)

Two viable designs; the gate decides. Recommendation: **Option A** (lean, matches D-014's
"promotion means copying into the repo"; Option B stays available if multi-project use demands it).

## Option A — render project-layer knowledge only (recommended)

- **`src/render.rs`** — the instruction-file pointer list includes only `Layer::Project` knowledge.
  Global-only entries never enter committed output. (Global-layer code path and the now-unused
  layer match arm removed.)
- **`src/commands/check.rs`** — new warning per global-only knowledge entry:
  "global-only knowledge `<path>` is not rendered into committed surfaces; copy it into
  `.base/canon/knowledge/` to adopt it in this project" — dropped honestly, never silently.
- **`docs/SPEC.md` §5** — amend the knowledge bullet: the global canon is the personal library;
  promotion is `run artifact → project knowledge/ → global canon`; an *existing* project adopts a
  lesson by copying it into its own `.base/canon/knowledge/` (one file copy, then sync). DoD 7
  reads "visible in another project after adoption + sync".
- **`docs/DECISIONS.md`** — D-017 records the choice, names Option B as the future upgrade path,
  and notes the general hazard (rules/agents/pipelines).
- **New work item W-0005** — global-only rules/agents/pipelines still render into committed
  output; decide guard or vendoring for those kinds (out of W-0004's criteria).
- **`tests/cli.rs`** — new test: global-only knowledge entry → sync output contains no pointer to
  it and is byte-identical with and without the global canon present; `check --json` lists the
  warning.

## Option B — vendor global knowledge into the repo on sync

`sync` copies global knowledge into a manifest-owned repo path (e.g. `.base/knowledge/global/`),
renders pointers to those repo paths, and the loader prefers vendored copies so output stays
repo-derivable; machines with a *stale* global canon get an honest drift signal. Preserves the
spec's "reaches every project on next sync" literally. Costs: a new knowledge source layer in the
loader, vendor/removal semantics in sync, more tests — machinery for a multi-project reality that
doesn't exist yet.

## Verification (either option)

1. Re-run the reproduction: promote lesson to scratch global → sync clone → `sync --check` with no
   global must stay green (the failing step from diagnosis now passes).
2. `cargo fmt` / `clippy -D warnings` / full suite; `base sync` + `sync --check` in this repo.
3. Rebuild + reinstall release binary; capture before/after in `evidence/`.

## Risks

- Option A changes a spec promise (§5 sentence, DoD 7 wording) — that is the point of gating this
  decision; the promotion loop survives with one added explicit copy step.
- This repo's own instruction files currently contain no global-only pointers (fully shadowed), so
  no committed-surface churn is expected here beyond none; the fix is behavioral for future promotions.

## Delivery

Branch `fix/w-0004-global-knowledge`; run artifacts + history line; PR to `main`.
