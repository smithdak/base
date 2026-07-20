---
id: session-context
description: Inject the active work item and durable handoff at session start.
event: session-start
mode: context
command:
  - base
  - state
  - context
timeout-seconds: 10
---

At session start, run `base state context` before acting. If the target has no native lifecycle
hook, load the current work item and `.base/state/handoff.md` manually.
