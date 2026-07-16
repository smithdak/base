# Plan: Adopt the Sitecore pack into sitecoreai

All steps run in `D:\github\sitecoreai` unless noted. The base repo only receives run
artifacts, the W-0009 status change, and the ledger line.

## Steps

1. **Branch.** `git checkout -b chore/adopt-base-sitecore-pack` from `main`. Unrelated
   working-tree modifications ride along uncommitted and are never staged.
2. **Scaffold.** `base init` — creates `.base/` (base.toml with default gates, starter canon,
   work/runs/knowledge state folders). Touches nothing outside `.base/`.
3. **Port the hand-authored CLAUDE.md** (the merge story): move its content into
   `.base/canon/rules/sitecoreai-repo.md` with `id: sitecoreai-repo` frontmatter — structure,
   stack, component pattern, Content SDK imports, git workflow, local dev, do-not-edit list,
   verbatim apart from frontmatter. Then `git rm CLAUDE.md` (upstream's 1333-line original
   stays in git history; the 52 hand-written lines now live in canon).
4. **Copy the pack** per `~/.base/canon/packs/sitecore/pack.md`:
   - `rules/sitecore-conventions.md` → `.base/canon/rules/`
   - `knowledge/sitecore-{content-modeling,platform-decisions,governance-operability}.md`
     → `.base/canon/knowledge/`
   - Append one routing line per knowledge file to `.base/canon/knowledge/INDEX.md`, matching
     its existing link format and each file's frontmatter description.
5. **Compile.** `base sync` — generates `CLAUDE.md`, `AGENTS.md`, `.claude/` (agents, skills,
   settings.json with gate hooks), `.codex/`, `.github/copilot-instructions.md`,
   `.github/prompts/`, `.agents/skills/`.
6. **Verify** (evidence captured to this run's `evidence/`):
   - `base check` passes; enforcement matrix reported.
   - Generated `CLAUDE.md` contains the ported repo conventions and pack conventions.
   - `git status --short` shows only intended paths staged; component-map/next-env changes
     remain unstaged.
7. **Commit** everything from steps 2–5 as one commit on the feature branch. No push.
8. **Record** (base repo): friction notes for W-0010's trigger and any pack gaps →
   `evidence/adoption-friction.md`; `result.md`; move W-0009 to review; append one
   `history.jsonl` line (`outcome: completed`); commit base-side artifacts on a branch and
   open a PR.

## Risks

- **CLAUDE.md replacement** is the one destructive-looking move: the uncommitted hand-authored
  file is deleted in step 3 *after* its content is ported verbatim to canon in the same step.
  If sync output later drops any of it, the canon rules file still holds the source.
- **Rule granularity**: porting the whole 52 lines as one always-on rules file makes compiled
  CLAUDE.md carry everything the old file did — same behavior as today, deliberately; splitting
  into knowledge can happen later inside sitecoreai.
- **Upstream drift**: upstream also ships root-level `copilot-instructions.md` and its own
  huge CLAUDE.md upstream; future pulls may conflict on CLAUDE.md. Accepted — this repo is a
  local test bed and the conflict resolution is "keep generated".
- **Gap flow-back** (W-0009 criterion 3) happens only if adoption exposes pack-content gaps;
  edits then land in `~/.base/canon/packs/sitecore/` as authored changes, noted in result.md.

## Verification commands

- `base check` (sitecoreai) → `evidence/base-check.txt`
- `grep` for ported + pack rule sentinel lines in generated CLAUDE.md → `evidence/claude-md-content.txt`
- `git -C D:/github/sitecoreai log --oneline -1 --stat` and `git status --short` → `evidence/commit.txt`
