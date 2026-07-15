---
id: W-0006
title: Add research, writing, and automation pipeline families
status: todo
verdict: pending
created: 2026-07-15
tags:
- pipelines
- design
---

# Add research, writing, and automation pipeline families

D-002 names research, writing, and automation as the pipeline families that must someday
land with zero core changes — this item makes that promise concrete. Each family is a pure
canon addition: new pipelines and stages under `.base/canon/pipelines/`, no new core types
or CLI verbs. Doubles as the proof that the core is genuinely domain-neutral. Related:
[[W-0007]] project-type packs may bundle these pipelines for specific project types.

## Acceptance Criteria

- [ ] Canon defines at least one pipeline for each family: research, writing, automation
- [ ] Zero core changes are required — the new families are pure canon additions, proving D-002's domain-neutral commitment
- [ ] Each new pipeline compiles via base sync to all active targets and has at least one real run recorded in history.jsonl
