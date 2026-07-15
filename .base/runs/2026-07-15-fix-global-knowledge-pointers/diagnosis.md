# Diagnosis: global knowledge pointers poison committed output (W-0004)

## Reproduction (evidence/repro-before.md)

Copying one lesson into a (scratch) global canon and syncing a clean clone rewrote all three
instruction files with `- global canon ` + backtick-quoted `tokenize-before-you-judge.md` — a pointer whose
source lives outside the repo. Re-checking the same repo without that global canon:
`sync --check` fails with `content differs` on `CLAUDE.md`, `AGENTS.md`, and
`.github/copilot-instructions.md`. Exit 1, reproduced deterministically.

## Root cause — `render_instructions`, src/render.rs

The knowledge pointer list iterates the merged `canon.knowledge` map and renders **both layers**,
labeling global entries "global canon `path`". Because project entries shadow same-path global
entries, every Layer::Global entry left in the merged map is by definition *global-only* — i.e.,
its source is guaranteed absent from the repo. The renderer therefore commits references it cannot
reproduce from the repo, violating D-014 ("never the sole source of a committed surface").

## Blast radius

Wider than knowledge: a global-only **rule** renders its whole body into instruction files, and a
global-only **agent or pipeline** emits entire generated files — same trap, larger payload. The
walking skeleton never hit this because `init --global` seeds identical files that the project
layer fully shadows. W-0004's criteria scope this run to knowledge; the general case needs its own
work item rather than a scope-widened fix here.

## Constraint on any fix

The spec currently promises "global knowledge reaches every project on next sync" (§5). That
promise and D-014 cannot both hold via ambient render-time merging; either the promise is amended
(promotion = explicit copy into each adopting repo) or the mechanism changes (sync vendors global
knowledge into manifest-owned repo files and renders repo paths). Both keep DoD 7 achievable.
