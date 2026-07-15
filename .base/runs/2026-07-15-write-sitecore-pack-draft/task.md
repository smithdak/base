# Task: draft the Sitecore project-type pack (writing, feeds W-0007)

## Outcome

The first project-type pack exists as adoptable canon files: a manifest plus the rule and
knowledge entries named by the research run's inventory
(`.base/runs/2026-07-15-research-sitecore-pack-contents/findings.md`). Delivered to the global
canon library at `~/.base/canon/packs/sitecore/`, with working copies retained in this run folder
and the inventory attached to the W-0007 work item as committed evidence.

## Constraints

- Every pack file is canon-shaped: YAML frontmatter (`id`, `description`) + prose, lowercase
  kebab-case ids, vendor-neutral — adoptable into any project's `.base/canon/` by plain copy.
- Distill, don't copy (D-019): content is an authored rewrite of the source skills, scoped to
  what a Sitecore *project* needs resident; no skill bodies duplicated wholesale.
- Pack scope follows the research findings exactly: one rule file, three knowledge files, no
  pipelines, no agents.
- Zero repo-surface impact: `packs/` is outside the compiler's kind folders, so `base sync`
  output in this repo must not change.

## Assumptions

- Destination `~/.base/canon/packs/sitecore/` per the findings' Q2; D-020 will record this as
  the pack home.

## Acceptance checks

1. Outline approved before drafting (plan-approval gate).
2. All five files exist at the destination with valid frontmatter; working copies in the run
   folder match.
3. `pack.md` carries the inventory and adoption instructions; a copy of the inventory lands in
   `.base/work/W-0007-project-type-packs/`.
4. `base sync --check` still passes in this repo (no surface change).
