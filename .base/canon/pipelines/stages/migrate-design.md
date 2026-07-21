---
id: migrate-design
description: Design the optimal base-native architecture, then stop for human approval.
---

Design the target, do not port the source. Follow `knowledge/migration-architecture.md`. First state
the real capabilities the system delivers (stripped of fragmentation — a `security-*` family is one
capability, not eight). Then design the minimal base-native architecture that delivers them: a small
set of role-based agents (analyst / implementer / reviewer + only genuinely distinct specialists),
capabilities as gated pipelines with verifiers, reusable techniques as skills, durable domain facts
as knowledge, with base-native names. Write an explicit **consolidate / rename / drop** list with the
reasoning: what merges, what is renamed, and what (runtime state, generated output, bespoke tooling)
is deliberately not migrated. This is a plan-approval gate: the human approves the architecture before
anything is authored.
