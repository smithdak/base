---
id: migrate-plan
description: Propose the canon mapping and the base improvements, then stop for human approval.
---

From the inventory, write the migration plan: the pack id, and for every source artifact the canon
kind it becomes (agent, skill, pipeline, policy, rule, knowledge, gate) or the honest reason it stays
out of canon. Resolve each judgment the report flags — skill versus pipeline, which `CLAUDE.md` prose
is a durable rule versus reference knowledge, which deny rule becomes a standing-denial gate, which
Claude-only knobs are dropped and why. Name the improvements base adds on top of a faithful
reproduction (see `knowledge/migration-mapping.md`). This is a plan-approval gate: do not author the
pack until the human approves the mapping.
