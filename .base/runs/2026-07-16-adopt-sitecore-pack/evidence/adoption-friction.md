# Adoption friction — evidence for D-020's `base adopt` trigger (W-0010)

First manual copy-adoption of a pack (sitecore → sitecoreai, 2026-07-16). Observed effort,
step by step:

| Step | Effort | Pack-copy work? |
|---|---|---|
| `base init` | one command | no — first-time overlay setup |
| Port pre-existing CLAUDE.md into canon rules | the bulk of the work: 52 lines moved verbatim + frontmatter, then `git rm -f` | no — merge story for a previously hand-managed repo |
| Copy pack rules file | one `cp` | yes |
| Copy three pack knowledge files | one `cp` | yes |
| INDEX.md routing lines | hand-composed 3 lines from each file's frontmatter description | yes — the only genuinely automatable tedium |
| `base sync` + `base check` | two commands | no |
| Selective staging (exclude `settings.local.json`, unrelated edits) | one careful `git add` | no — init/sync concern |

## Finding

**The copy itself was not tedious**: two `cp` commands and three hand-written index lines.
Everything that took real care was first-time-onboarding work (`init`, the CLAUDE.md merge
story, staging hygiene) that a `base adopt <pack>` helper would not touch — a helper only
automates the two `cp`s and possibly the INDEX lines.

On n=1, D-020's trigger ("copy-adoption proves tedious") **did not fire**. W-0010's own body
says that in this case it closes as won't-do with the finding recorded — that closure is
Dakota's verdict to record, not the agent's; this file is the finding.

## Pack gaps found (W-0009 criterion 3)

One: `pack.md`'s adoption instructions assumed a base-managed project — no mention of
`base init`, nor of porting pre-existing hand-authored harness files before the first sync
(`base sync` refuses to overwrite unowned files). Flowed back 2026-07-16 as an authored step 0
in `~/.base/canon/packs/sitecore/pack.md`. No content gaps in the rules/knowledge files
surfaced during this adoption; deeper content verdicts need real Sitecore work sessions in the
adopted repo.
