---
id: automation
description: Turn a repetitive manual procedure into a proven, repeatable mechanism.
stages:
  - use: intake
  - use: plan
    gate: plan-approval
  - use: execute
  - use: verify
  - use: record
---

Use this pipeline when the deliverable is a mechanism that will run again — a script, hook, or
scheduled job replacing a manual procedure. Verification is not satisfied by reading the code:
the mechanism must run end to end at least once with its real output captured under `evidence/`.
An automation that has never run is a draft, not a deliverable.
