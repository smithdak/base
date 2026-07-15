---
id: W-0003
title: Hook false-positives on quoted text mentioning git push
status: done
verdict: pass
created: 2026-07-15
tags:
- gates
- bug
---

# Hook false-positives on quoted text mentioning git push

## Acceptance Criteria

- [x] The exact W-0002 commit+push compound that was wrongly denied is permitted
- [x] Quoted text containing git push and main-like substrings (remains, domain) is not denied
- [x] Real pushes stay denied, including no-space separators like x&&git push origin main
