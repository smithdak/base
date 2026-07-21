---
id: delivery-foundation
description: Prove canonical composition and generated target surfaces are valid and drift-free.
checks:
  - id: canon
    run: [base, check, --json]
    timeout-seconds: 120
  - id: generated-surfaces
    run: [base, sync, --check]
    timeout-seconds: 120
---

This portable suite proves the Base operating-model layer only. It never substitutes for the
project's behavior-specific build, test, security, infrastructure, or release verification.
