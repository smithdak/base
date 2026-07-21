---
id: delivery
description: Discover, approve, implement, prove, review, and record a software change.
stages:
  - use: delivery-discover
    agent: delivery-analyst
  - use: delivery-plan
    gate: plan-approval
    agent: delivery-analyst
  - use: delivery-implement
    agent: delivery-implementer
  - use: delivery-prove
    agent: delivery-implementer
    verifier: delivery-foundation
  - use: delivery-review
    agent: delivery-auditor
    independent-review: true
  - use: record
---

Use for consequential repository work that must survive handoffs and preserve an evidence trail.
The attached `delivery-foundation` suite proves Base composition and generated-surface integrity.
The project must also define and run the behavior-specific verifier suite named in the plan; a
verifier is an executable contract, not a prose checklist.
