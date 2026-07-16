# Result: Adopt the Sitecore pack into sitecoreai (W-0009)

Outcome: **completed**. All four task acceptance checks pass.

## Changed paths

- **sitecoreai** — commit `0f3069d` on branch `chore/adopt-base-sitecore-pack` (30 files,
  never pushed): `.base/` overlay (base.toml, starter canon, pack copies
  `rules/sitecore-conventions.md` + three `knowledge/sitecore-*.md`, INDEX routing lines,
  ported `rules/sitecoreai-repo.md`), regenerated surfaces (CLAUDE.md, AGENTS.md, `.claude/`,
  `.codex/`, `.agents/`, `.github/copilot-instructions.md`, `.github/prompts/`), old
  hand-authored CLAUDE.md removed after verbatim port.
- **global library** — `~/.base/canon/packs/sitecore/pack.md`: authored step 0 added to the
  adoption instructions (init-first + port pre-existing harness files), the one gap found.
- **base repo** — this run folder; W-0009 moved to review.

## Verification

| Check | Verdict | Evidence |
|---|---|---|
| `base check` passes in sitecoreai; fidelity matrix reported (both gates enforced on claude) | pass | `evidence/base-check.txt` |
| Generated CLAUDE.md carries ported repo conventions + pack conventions; INDEX routes 3 knowledge files | pass | `evidence/claude-md-content.txt` |
| One adoption commit on a feature branch; unrelated modifications unstaged; nothing pushed | pass | `evidence/commit.txt` |
| Friction evidence captured for W-0010's trigger; pack gaps flowed back | pass | `evidence/adoption-friction.md` |

## Limitations

- Friction finding rests on n=1: the copy was not tedious, so D-020's trigger for `base adopt`
  did not fire. W-0010's recorded intent says that closes it as won't-do — Dakota's verdict to
  record.
- Pack *content* quality is untested by adoption alone; it gets exercised when real Sitecore
  work runs in the adopted repo (the rest of W-0009's knowledge loop).
- sitecoreai's adoption commit lives only locally (third-party upstream); merging it into that
  repo's local main is Dakota's call.
