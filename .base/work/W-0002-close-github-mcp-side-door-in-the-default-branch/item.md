---
id: W-0002
title: Close GitHub MCP side door in the default-branch gate
status: todo
verdict: pending
created: 2026-07-15
tags:
- gates
- security
---

# Close GitHub MCP side door in the default-branch gate

## Acceptance Criteria

- [ ] Claude adapter mechanically blocks GitHub MCP tools from writing to the default branch (PreToolUse hook inspects tool arguments, mirroring the git-push binding)
- [ ] base check fidelity notes honestly cover the MCP surface
- [ ] Feature-branch writes and PR merges via MCP remain permitted
