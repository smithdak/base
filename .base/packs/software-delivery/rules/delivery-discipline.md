---
id: delivery-discipline
description: Evidence-led delivery rules for consequential repository changes.
---

- Establish the current work item, intended outcome, and acceptance checks before implementation.
- Commit both a new work-item folder and its reported `.base/work/.ids/W-NNNN` reservation; stage
  the whole `.base/work/` change so cross-branch duplicate IDs cannot merge silently.
- Separate repository-local proof from unavailable infrastructure or production proof.
- Treat failed and inconclusive verification as distinct non-passing outcomes.
- Preserve a handoff whenever work stops with a safe next action still outstanding.
- Record architecture decisions that constrain future implementation, not transient tactics.
