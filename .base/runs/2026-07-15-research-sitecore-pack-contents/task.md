# Task: what should the first Sitecore project-type pack contain? (research, feeds W-0007)

## Outcome

A cited answer to two questions W-0007 leaves open: (1) which canon definitions — rules,
knowledge, pipelines, agents — earn a place in the first Sitecore pack, and (2) where pack
sources should live so adoption honors seed-and-adopt semantics. The findings are concrete
enough for a writing run to draft the pack from them directly.

## Constraints

- Pack contents must be canon-shaped: vendor-neutral Markdown with frontmatter, adoptable into a
  project's `.base/canon/` by copy (D-017/D-018). Nothing harness-specific.
- Zero core changes (D-002): the answer may not require new canon kinds, CLI verbs, or schema.
- Respect D-019's boundary: portable personal technique belongs in skillsmith/user-level skills;
  a pack holds project-resident ways of working. The pack must not duplicate skill bodies.
- Read-only run: findings that demand action become work items or decision entries, not edits.

## Assumptions

- Dakota's existing Sitecore know-how (user-level `sitecore-architect` skill, installed
  `sitecore` developer skill) is the best available proxy for what a Sitecore project needs
  agents to know on every turn versus on demand.

## Acceptance checks

1. `sources.md` records every consulted source with location and weight.
2. `findings.md` answers both questions with each material claim tied to a source, states
   confidence, and names what would change the conclusions.
3. The proposed inventory maps every entry to an existing canon kind.
