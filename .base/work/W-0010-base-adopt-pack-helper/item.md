---
id: W-0010
title: base adopt <pack> helper
status: review
verdict: pending
created: 2026-07-16
tags:
- packs
- cli
---

# base adopt <pack> helper

D-020 records this helper as the upgrade path for pack adoption "when copy-adoption proves
tedious." That trigger cannot have fired yet: no project has performed a manual adoption
(W-0009 is the first). This item exists so the candidate sits on the board instead of only in
decision prose — it is deliberately sequenced *after* W-0009, whose experience either supplies
the tedium evidence or shows the manual copy is fine as-is (in which case this item closes as
won't-do with that finding recorded).

Scope guardrails from D-020: `packs/` never becomes a compiler concept; the helper is a copy
convenience only, and the adopting repo's history still shows a visible copy.

## Acceptance Criteria

- [ ] base adopt <pack> copies a global-library pack's files into the project's .base/canon/, refusing to overwrite existing files, and prints the manual follow-ups it cannot do (INDEX.md routing lines, sync, commit)
- [ ] Adoption stays a visible copy in the adopting repo's history (D-018/D-020); the helper adds zero compiler concepts
- [ ] Sequencing honored: built only after the first manual adoption (W-0009, done 2026-07-16). That adoption found the copy itself low-tedium (`2026-07-16-adopt-sitecore-pack/evidence/adoption-friction.md`); Dakota directed the build anyway on 2026-07-16, superseding D-020's tedium trigger — the helper also encodes the adoption protocol so it is not re-derived from pack.md each time
