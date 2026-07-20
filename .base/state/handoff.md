---
work-item: W-0011
run: 2026-07-20-operating-model-core
---

# Handoff

## Outcome sought

Promote Base into the cross-harness operating-model core for Claude Code, Codex, and GitHub Copilot.

## Current state

Implementation is complete locally on `codex/operating-model-core` and W-0011 is ready for human
review. Base 0.2.0 is installed on `PATH`; the global `software-delivery` 1.2.0 pack was refreshed
without replacing the existing Sitecore pack.

## Evidence

- Canonical verifier: `evidence/verifications/base-20260720T211559.971Z-p25712.json` — pass.
- Test inventory: 39 unit, 45 CLI integration, and 1 specification test — all pass.
- Installed hook probes denied default-branch pushes for Claude, Codex, and Copilot, denied the live
  Copilot `github-mcp-server-push_files` shape, allowed GitHub MCP reads, and injected W-0011/run
  context for all three target protocols.

## Risks and unknowns

- Linux execution, a live Claude hook session, Codex hook trust/execution, and post-fix Copilot cloud
  execution remain unverified runtime cells.
- Repository hooks are workflow guardrails, not authorization; server-side branch protection remains
  authoritative, and Copilot host timeouts are fail-open.

## Next action

Review W-0011 and record the human verdict with
`base work move W-0011 done --verdict pass|fail`.
