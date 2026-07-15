---
id: sitecore
description: Project-type pack for Sitecore repositories — conventions and platform reference.
---

# Sitecore pack

Seed canon for repositories building on Sitecore (XM Cloud, XM/XP, SXA, headless). Adopting it
gives every harness the always-on conventions a Sitecore codebase demands plus on-demand
platform reference. It deliberately excludes pipelines and agents: the generic families fit
Sitecore work, and architect-level technique stays in user-level skills (D-019).

## Inventory

| File | Kind | Covers |
|---|---|---|
| `rules/sitecore-conventions.md` | rules | always-on constraints: serialization, Standard Values, Helix, secrets, naming |
| `knowledge/sitecore-content-modeling.md` | knowledge | template hierarchy, Standard Values checklist, field conventions |
| `knowledge/sitecore-platform-decisions.md` | knowledge | XM Cloud vs. XP/XM signals, topology, migration phasing |
| `knowledge/sitecore-governance-operability.md` | knowledge | governance, security hardening, CI/CD, observability/DR |

## Adoption

1. Copy `rules/sitecore-conventions.md` into the project's `.base/canon/rules/`.
2. Copy the three `knowledge/` files into the project's `.base/canon/knowledge/` and add one
   routing line per file to that project's `knowledge/INDEX.md`.
3. Run `base sync`; commit the copied canon and regenerated surfaces together.

Adoption is a visible copy (D-017/D-018): the project owns its copy from that commit on, and
drift from this library copy is deliberate, not an error.

## Provenance

Distilled 2026-07-15 from Dakota's `sitecore-architect` user skill and the installed
`plugin-cms-toolkit` `sitecore` skill (v1.0.1) — authored rewrite, not a copy; see
`base` repo run `2026-07-15-research-sitecore-pack-contents` for the selection rationale.
