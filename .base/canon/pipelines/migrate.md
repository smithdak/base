---
id: migrate
description: Understand another agent system and design a better base-native one.
stages:
  - use: migrate-inventory
    agent: delivery-analyst
  - use: migrate-design
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

Use to migrate a client or legacy system (agents, skills, commands, hooks, workflows) into Base.
Assume the source is messy and over-built. The goal is **not a mirror** — it is to understand what
the system actually does and design a better one using Base's methodology: capability-first, a
minimal set of role-based agents, capabilities as gated pipelines with verifiers, domain expertise
as knowledge, base-native naming. `base ingest` understands the source and surfaces its
fragmentation; the human approves an architecture; the agent authors the pack. Load
`knowledge/migration-architecture.md` before designing.
