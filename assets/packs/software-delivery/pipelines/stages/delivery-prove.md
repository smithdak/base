---
id: delivery-prove
description: Execute the project verification contract and retain typed, hash-backed results.
---

Run the behavior-specific project verifier named in the approved plan with
`base verify <suite> --run <run-slug>`. The pipeline also attaches the generic
`delivery-foundation` verifier for Base composition and generated-surface integrity. Classify each
check as pass, fail, or inconclusive. Record unavailable infrastructure and external gates
separately from local proof; do not collapse them into a pass.
