---
id: build
description: Plan, approve, implement, verify, and record a software change.
stages:
  - use: intake
  - use: plan
    gate: plan-approval
  - use: execute
  - use: record
---

Use this pipeline for repository changes that should leave an auditable plan, result, and history
entry. Treat the user's invocation text as the task; do not invent a separate objective.
