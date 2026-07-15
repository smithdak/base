# Plan: approval-as-artifact gate satisfaction

## Protocol design

A stage-approval gate may declare `satisfied-by = "approvals/<gate-id>.md"` (path relative to
the run folder). The compiled gate prose then instructs the agent: after completing the gated
stage, write `<satisfied-by>.request` describing what needs approval, then stop. The human
records the verdict from their own terminal: `base approve <run> <gate> [--deny] [--note ...]`
writes the response artifact (stamped: gate, run, verdict, by, at, note) and refuses to
overwrite one that exists. A request with no response is *pending*; the Claude hook denies all
mutating tools while any run has a pending request. Either verdict lifts the mechanical block;
the prose routes a denial to `record` with outcome `aborted`. Request/response derivation is
mechanical (`<path>.request` ↔ `<path>`), no parsing.

## Changes

1. **`src/config.rs`** — `Gate.satisfied_by: Option<String>` (`satisfied-by` in TOML, omitted
   when none). Validate: only on stage-approval gates; relative path; no `..`; non-empty.
2. **`src/cli.rs`** — new visible verb `Approve(ApproveArgs)`: positional `run` (slug) and
   `gate` (id); flags `--deny`, `--by <WHO>` (default: git config user.name, else env
   USERNAME/USER), `--note <TEXT>`.
3. **`src/commands/approve.rs`** (new) — resolve gate (must exist, stage-approval, with
   `satisfied-by` defaulting to `approvals/<gate-id>.md`), resolve run folder under
   `.base/runs/`, refuse existing response, write stamped Markdown record, `--json` support.
4. **`src/commands/hook.rs`** — pending-request scan: `.base/runs/*/…/*.request` without the
   derived response file → deny with a reason naming the run, gate, and the approve command.
   Refactor decision logic to take the project root explicitly for unit testing. Standing
   denial check unchanged and still first.
5. **`src/render.rs`** —
   - `enforcement()`: stage-approval + claude + `satisfied_by` present → `enforced`.
   - Gate prose in `render_pipeline`: for `satisfied-by` gates, replace the trust-me STOP with
     the artifact protocol (request file, approve command, standing approvals must be recorded
     with `--note` citing the source, denied → record aborted). Gates without `satisfied-by`
     keep today's prose byte-for-byte.
   - `.claude/settings.json`: emitted when the standing denial *or* an artifact gate is active.
     With an artifact gate: three matcher groups — `Bash` (drops the `if: git push` filter so
     the pending check sees every command), `Edit|Write|NotebookEdit`, `mcp__github__.*` — all
     calling the same `base __hook claude-pre-tool`. Without one: today's shape unchanged.
6. **`src/commands/check.rs`** — claude note mentions artifact-gate enforcement when active;
   the existing PATH-resolution downgrade warning applies to the new `enforced` cell too
   (already generic over `fidelity == "enforced"`).
7. **`.base/base.toml`** — `plan-approval` gains `satisfied-by = "approvals/plan-approval.md"`.
8. **`docs/SPEC.md` §7** — add the `base approve` row, "five verbs" → "six verbs" (tether).
9. **`docs/DECISIONS.md`** — D-021 recording the protocol, the whole-session pending block
   trade-off, and the honest per-target fidelity.
10. **`base sync`** — regenerate all surfaces (gate prose changes ripple into build/fix/
    writing/automation skills × 3 targets, settings.json, instruction files); commit generated
    output with the code.

## Verification

- Unit: config round-trip + rejection cases; hook pending/answered/absent × Bash/Edit/mcp
  events (tempdir project root); approve success/duplicate/unknown-gate/unknown-run; render
  enforcement cells and settings.json shape; existing hook tests stay green.
- Integration: `cargo fmt --check`, `clippy -D warnings`, full `cargo test` (spec tether
  proves SPEC §7 ↔ clap), `base sync` + `sync --check`, `base check` matrix shows
  plan-approval `enforced|assisted|advisory` across claude|codex|copilot.
- End-to-end (evidence/): synthetic run folder → pipe PreToolUse JSON to the hook with a
  pending request (expect deny) → `base approve` → same event (expect allow) → duplicate
  approve (expect refusal). Captured to `evidence/hook-loop.md`.
- `cargo install --path .` last, so the live hook binary matches; note: enforcement activates
  for future sessions the moment a request file is pending — synthetic test folders are
  cleaned up inside the run.

## Risks

- Hook now runs on every Bash/Edit/Write call: scan must stay cheap (single directory walk of
  `.base/runs/`; no run folders → no cost) and fail open on IO errors to avoid bricking
  sessions on filesystem oddities — standing denial keeps its fail-closed posture.
- `settings.json` shape change must not break Claude Code parsing; validated by loading the
  file in this session after sync.
- Whole-session block on any pending request can surprise (e.g. stale request from an aborted
  run left on disk). Mitigation: deny reason names the exact file and both ways to clear it.

## Gate: plan-approval

Awaiting explicit approval in conversation — the last plan approved under the prose-only
regime, if this plan lands.

Approved in conversation ("continue"), 2026-07-15.
