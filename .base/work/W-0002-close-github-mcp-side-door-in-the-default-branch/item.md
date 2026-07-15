---
id: W-0002
title: Close GitHub MCP side door in the default-branch gate
status: done
verdict: pass
created: 2026-07-15
tags:
- gates
- security
---

# Close GitHub MCP side door in the default-branch gate

## Acceptance Criteria

- [x] Claude adapter mechanically blocks GitHub MCP tools from writing to the default branch (PreToolUse hook inspects tool arguments, mirroring the git-push binding)
- [x] base check fidelity notes honestly cover the MCP surface
- [x] Feature-branch writes and PR merges via MCP remain permitted
