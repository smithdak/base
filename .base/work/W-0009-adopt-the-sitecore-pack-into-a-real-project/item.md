---
id: W-0009
title: Adopt the Sitecore pack into a real project
status: review
verdict: pending
created: 2026-07-16
tags:
- packs
- knowledge
---

# Adopt the Sitecore pack into a real project

The Sitecore pack (D-020, drafted via `2026-07-15-write-sitecore-pack-draft`) sits unused in
`~/.base/canon/packs/sitecore/`. Adopting it into a real Sitecore repository is what completes
the knowledge loop the pack exists for — and it is the *first* manual copy-adoption, so it also
produces the tedium evidence D-020 names as the trigger for W-0010.

**Target triage (2026-07-16):** the only real Sitecore codebase on this machine is
`D:\github\sitecoreai` — a clone of `Sitecore/xmcloud-starter-js` (XM Cloud authoring solution
plus Next.js example) with active local work. Two blockers make target selection a human call:

1. That repo already has a hand-modified `CLAUDE.md` and a live `.claude/` folder; `base init`
   plus `base sync` would regenerate `CLAUDE.md` and could clobber that work. Adoption there
   needs a merge story for the existing harness files first.
2. It tracks an upstream third-party remote (`origin = Sitecore/xmcloud-starter-js`), so
   committed base surfaces would live only on a local branch — fine for a test bed, but Dakota
   should confirm it counts as the "real project", or name a client/work repo instead.

## Acceptance Criteria

- [ ] A real Sitecore repository adopts the pack per pack.md: rules and knowledge files copied into its .base/canon/, INDEX.md routing lines added, base sync run, copies and regenerated surfaces committed together
- [ ] The adoption experience is recorded: friction observed (or absence of it) lands in the ledger or a run artifact, providing the evidence D-020 names as the trigger for a base adopt helper
- [ ] Any pack-content gaps found during adoption flow back into the global library copy as authored edits, completing the knowledge loop
