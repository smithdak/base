---
id: migrate-prove
description: Prove the pack validates, adopts cleanly, and composes without drift.
---

Prove the authored pack. Run `base pack check <path>`, then adopt it into a scratch project and run
`base check` and `base sync --check` there; the attached `delivery-foundation` verifier proves Base
composition and generated-surface integrity. Classify each result as pass, fail, or inconclusive, and
retain the evidence under the run. A pack that does not adopt cleanly is not done — record the
non-passing outcome rather than narrating success.
