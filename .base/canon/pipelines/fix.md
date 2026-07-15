---
id: fix
description: Diagnose, plan, approve, repair, and prove a defect fix.
stages:
  - use: intake
  - use: diagnose
  - use: plan
    gate: plan-approval
  - use: execute
  - use: verify
  - use: record
---

Use this pipeline when something is broken and the cause is not yet established. The diagnosis
bounds the plan: do not plan past what the reproduction demonstrates, and do not widen the repair
beyond the diagnosed cause.
