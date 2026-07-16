# Task: Adopt the Sitecore pack into sitecoreai (W-0009)

Date: 2026-07-16 · Pipeline: build · Harness: claude · Work item: W-0009

## Outcome

`D:\github\sitecoreai` (clone of `Sitecore/xmcloud-starter-js`, Dakota's active Sitecore test
bed — target confirmed by Dakota 2026-07-16) becomes a base-managed project that has adopted
the Sitecore pack from `~/.base/canon/packs/sitecore/` per the pack's own adoption
instructions: pack rules and knowledge copied into its `.base/canon/`, INDEX routing lines
added, `base sync` run, copies and regenerated surfaces committed together on a feature
branch. This is the first manual copy-adoption, so the run also captures the friction evidence
D-020 names as the trigger for W-0010 (`base adopt` helper).

## Constraints

- Preserve the hand-authored 52-line `CLAUDE.md` currently sitting **uncommitted** in
  sitecoreai: its content must survive as canon (it becomes a project rules file) before the
  file itself is replaced by generated output. Nothing hand-written may be lost.
- Do not commit sitecoreai's unrelated working-tree modifications
  (`examples/basic-nextjs/.sitecore/component-map*.ts`, `next-env.d.ts`).
- Never push: the default-branch gate applies, and origin is a third-party upstream
  (`Sitecore/xmcloud-starter-js`). The adoption lands as one local commit on a feature branch.
- Adoption is a visible copy (D-018/D-020): no new compiler concepts, no pack machinery.

## Assumptions

- `base` is on PATH (the base repo's own compiled hooks already invoke it successfully).
- `base init` in sitecoreai scaffolds the default overlay: `.base/base.toml` (targets
  claude/codex/copilot, gates plan-approval + never-push-default-branch), starter canon
  (working-agreements, builder/reviewer, pipeline families and stages, knowledge INDEX).
- `base sync` refuses to overwrite the unowned pre-existing `CLAUDE.md`
  (`src/commands/sync.rs:84`), so that file must be removed after its content is ported —
  the upstream 1333-line original stays recoverable in git history.
- No collisions on other generated paths: sitecoreai has no `AGENTS.md`, no
  `.claude/settings.json` (only `settings.local.json`), and upstream's root
  `copilot-instructions.md` is a different path from generated `.github/copilot-instructions.md`.

## Acceptance checks

1. `base check` passes in sitecoreai after adoption (canon valid, gate fidelity reported).
2. Generated `CLAUDE.md` in sitecoreai contains both the ported repo conventions and the pack's
   Sitecore conventions; `.base/canon/knowledge/INDEX.md` routes the three pack knowledge files.
3. `git log`/`git status` in sitecoreai show exactly one adoption commit on a feature branch,
   unrelated modifications left uncommitted, nothing pushed.
4. Friction observations (or their absence) are captured under `evidence/` for W-0010's trigger,
   and any pack-content gaps are noted for flow-back into the global library.
