# Task: approval-as-artifact gate satisfaction (W-0008)

## Outcome

Stage-gate approval stops being a prose STOP the agent is trusted to obey and becomes a durable
artifact protocol: a gate declares the artifact that satisfies it, `base approve` writes that
artifact with a stamp, and on Claude a compiled hook mechanically denies mutating tools while a
run's approval request is unanswered — upgrading plan-approval from `assisted` to `enforced` on
the reference harness, honestly reported per target.

## Constraints

- Gate declarations stay data (a path, a flag) — no conditions or expressions (D-008).
- The artifact protocol is harness-neutral prose; only enforcement is per-target, and the
  enforcement matrix must report each cell honestly (never overstate Codex/Copilot).
- Approval records are immutable: `base approve` refuses to overwrite an existing verdict.
- SPEC §7 is tethered (D-016): the verb table, the verb-count prose, and `tests/spec.rs` must
  move together with the new verb.
- Existing configs without `satisfied-by` keep exactly today's behavior (assisted, prose gate).
- The agent's own tool surface must not be able to satisfy the gate: while a request is
  pending, Bash / Edit / Write / NotebookEdit / GitHub-MCP calls are all denied, which also
  blocks self-running `base approve` and forging the response file. The human approves from
  their own terminal, outside the hooked session.

## Assumptions

- Hook processes run with cwd inside the project, so the run scan can use project discovery.
- Any pending request blocks the whole session's mutating tools (not just the gated run) —
  acceptable for one user in v1; recorded, not solved.

## Acceptance checks (from W-0008)

1. A decision (D-021) records approval-as-artifact semantics: gates declare `satisfied-by`,
   satisfaction is a durable file, never only conversation.
2. `base approve <run> <gate>` writes the stamped record (who, when, verdict, note); `--deny`
   records a denial; a second write is refused.
3. The Claude adapter compiles the enforcement: with a pending unanswered request, the hook
   denies mutating tool calls; with the response artifact present (either verdict), it allows.
   The matrix reports plan-approval as `enforced` on claude, `assisted` on codex, `advisory`
   on copilot.
4. Standing approvals are recorded artifacts citing their source (`--note`), demanded by the
   compiled gate prose.
5. `cargo fmt` / `clippy -D warnings` / full test suite (including the SPEC §7 tether) /
   `base sync` / `sync --check` all green; regenerated surfaces committed.
