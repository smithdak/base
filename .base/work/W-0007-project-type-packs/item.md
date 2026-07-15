---
id: W-0007
title: Project-type packs
status: todo
verdict: pending
created: 2026-07-15
tags:
- design
- packs
---

# Project-type packs

Discussed during pre-build planning and discovery (before 2026-07-14) but never recorded;
recovered from memory 2026-07-15. A **pack** is a curated bundle of canon definitions —
rules, knowledge, pipelines, agents — for a project type we work on repeatedly, adopted
into a project's `.base/canon/` overlay when that project matches the type. Sitecore is
the first candidate; other common project types can follow the same shape.

The architecture already anticipates this: packs are an overlay-content concept riding on
D-007 (project overlay) and D-017/D-018 (seed-and-adopt, adoption by visible copy). Per
D-002 they must require zero core changes. Open questions: where pack sources live (global
canon subfolder vs. separate repo vs. skillsmith-style library), and whether adoption is a
manual copy or a CLI helper (`base adopt <pack>`).

## Acceptance Criteria

- [ ] Pack concept is defined and recorded as a decision: a bundle of canon definitions (rules, knowledge, pipelines, agents) for a common project type, delivered into a project's .base/canon/ overlay
- [ ] Packs require zero core changes (D-002) and honor seed-and-adopt semantics (D-018) — no committed bytes sourced outside the repo
- [ ] A Sitecore pack is drafted as the first candidate, with an inventory of what it contains
