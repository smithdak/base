---
id: migrate
description: Ingest another agent system and author an improved, adoptable base pack.
stages:
  - use: migrate-inventory
    agent: delivery-analyst
  - use: migrate-plan
    gate: plan-approval
    agent: delivery-analyst
  - use: migrate-author
    agent: delivery-implementer
  - use: migrate-prove
    agent: delivery-implementer
    verifier: delivery-foundation
  - use: migrate-review
    agent: delivery-auditor
    independent-review: true
  - use: record
---

Use to migrate a client or legacy system (agents, skills, commands, hooks, workflows) into Base as a
reusable pack. The output is a library pack under `~/.base/canon/packs/<id>/` that reproduces what
the old system did and improves it by adding Base's operating model. `base ingest` models the source
deterministically and reports fidelity; the pack itself is an authored rewrite, never a byte copy
(docs/DECISIONS.md D-019/D-028). Load `knowledge/migration-mapping.md` before mapping.
