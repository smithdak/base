---
id: record
description: Record every pipeline exit in the project ledger.
---

Always run this stage, including after rejection, failure, or abort. Append exactly one compact JSON
object line to `.base/history.jsonl` with `slug`, `date`, `pipeline`, `harness`, `outcome`, and
`paths`. Use `completed`, `aborted`, or `failed` for `outcome`; never rewrite previous lines.
