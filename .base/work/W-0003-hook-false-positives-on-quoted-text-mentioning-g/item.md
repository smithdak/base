---
id: W-0003
title: Hook false-positives on quoted text mentioning git push
status: review
verdict: pending
created: 2026-07-15
tags:
- gates
- bug
---

# Hook false-positives on quoted text mentioning git push

## Acceptance Criteria

- [ ] The exact W-0002 commit+push compound that was wrongly denied is permitted
- [ ] Quoted text containing git push and main-like substrings (remains, domain) is not denied
- [ ] Real pushes stay denied, including no-space separators like x&&git push origin main
