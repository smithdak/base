# Findings: contents and home of the first Sitecore pack

## Q1 — What belongs in the pack

The selection rule that falls out of the constraints (sources 1, 2): a pack entry must be
something a Sitecore *project* needs resident in its repo — always-on constraints or on-demand
reference — not portable personal technique, which stays user-level per D-019. Applying that rule
to the two skill sources (3, 4) yields:

**rules/ — one file, `sitecore-conventions.md`.** The always-on subset both skills treat as
non-negotiable (sources 3 §governance, 4 §content-modeling): Helix/foundation-feature-project
boundaries respected in code and templates; every template gets Standard Values (insert options,
defaults, workflow, presentation); serialized items (SCS) are the source of truth — content and
schema changes land as serialization, never hand-made in a shared environment; secrets never in
serialized items or config; PascalCase field naming with `Is`/`Has` boolean prefixes. These are
constraints an agent must hold on *every* turn in a Sitecore repo, which is exactly the rules
kind (source 1, SPEC §3).

**knowledge/ — three files, routed by INDEX entries.** On-demand reference, each distilled (not
copied — D-019 requires authored rewrite) from the skill sources:
1. `sitecore-content-modeling.md` — template inheritance (base → feature → page), field naming
   and type selection, Standard Values checklist (source 4).
2. `sitecore-platform-decisions.md` — XM Cloud vs. XP/XM decision signals, topology cheatsheet,
   XP→XM Cloud migration phasing and its red flags (source 3).
3. `sitecore-governance-operability.md` — workflow/governance rules, security hardening list,
   CI/CD with serialization as pipeline stage, observability/DR expectations (source 3).

**pipelines/ — none in v1 of the pack.** The generic `build`/`fix` (and now `research`/`writing`/
`automation`) families already fit Sitecore work; a component-scaffold or upgrade pipeline is
speculative until a real Sitecore project demands it. Adding one now would violate the lean
principle (source 1, D-002 posture). Confidence: medium — first real project use may promote one
quickly.

**agents/ — none in v1 of the pack.** `builder`/`reviewer` suffice; a resident Sitecore-architect
agent would duplicate the user-level skill (source 3) across every project copy, the drift
hazard D-019 exists to prevent. Confidence: high.

## Q2 — Where pack sources live

**In the global canon: `~/.base/canon/packs/<pack-id>/`, mirroring the canon kind subfolders**
(`rules/`, `knowledge/`, `pipelines/`, `agents/`) plus a `pack.md` manifest (id, description,
inventory, adoption instructions). Rationale: D-018 already defines the global layer as a
seed-and-adopt library adopted by visible copy — packs are exactly that, just grouped by project
type instead of by kind. A separate repo or a skillsmith-style pipeline would add distribution
machinery the one-user system doesn't need (source 1, D-001/D-002). Adoption stays a manual copy
into the project's `.base/canon/` (then `base sync`); a `base adopt <pack>` helper is the
recorded upgrade path, deliberately deferred — same posture as the knowledge-promotion helper
(SPEC §10.4). Confidence: high for v1; multi-project tedium is the named trigger for the helper.

One consequence worth recording in the decision: `packs/` lives *outside* the kind folders the
compiler reads, so pack contents are invisible to `sync` until adopted — no new exclusion
reporting is needed and zero core changes hold (checked against source 1, D-018 mechanics).

## What would change these conclusions

- A real Sitecore project adopting the pack and immediately needing a scaffold pipeline →
  promote a `pipelines/` entry into the pack.
- A second or third pack (e.g. .NET service, Next.js app) making copy-adoption tedious →
  implement `base adopt`, per the W-0007 open question.
- `base check`/`sync` growing pack awareness would be a *new decision*, not implied by this one.

## Follow-ups

- D-020 decision entry recording the pack concept and home (this run's Q2).
- Writing run to draft `~/.base/canon/packs/sitecore/` from Q1's inventory.
