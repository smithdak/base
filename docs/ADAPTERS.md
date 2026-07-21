# Adapter surfaces and fidelity

Surface selection was verified against official product documentation on 2026-07-20. Harness
surfaces are volatile; re-verify this mapping before changing an adapter contract.

| Canon kind | Claude Code | Codex | GitHub Copilot |
|---|---|---|---|
| rules | `CLAUDE.md` | `AGENTS.md` | `.github/copilot-instructions.md` |
| agents | `.claude/agents/<id>.md` | `.codex/agents/<id>.toml` | `.github/agents/<id>.agent.md` |
| skills | `.claude/skills/<id>/` | `.agents/skills/<id>/` | `.agents/skills/<id>/` |
| pipelines | `.claude/skills/<id>/SKILL.md` | `.agents/skills/<id>/SKILL.md` | Agent Skill for CLI/cloud plus `.github/prompts/<id>.prompt.md` for VS Code |
| policies | `.claude/settings.json` | `.codex/hooks.json` | `.github/hooks/base.json` |
| default-branch guard | permission deny + `PreToolUse` | `PreToolUse` + project rules as defense in depth | `PreToolUse` repository hook |
| artifact stage gate | `PreToolUse` repository hook | trusted `PreToolUse` repository hook | `PreToolUse` repository hook |
| verifiers | `base verify` instruction | `base verify` instruction | `base verify` instruction |
| knowledge/state | instruction pointers + session context | instruction pointers + trusted session context | instruction pointers + session context |

Codex and Copilot share `.agents/skills/`; Base owns every shared generated path once. A shared
pipeline records the actual runtime (`codex` or `copilot`) instead of baking one target into the
other. Pipeline and standalone skill IDs therefore cannot collide.

## Fidelity vocabulary

`base check` reports an output profile and scope for every surface, plus one of these fidelity
values:

- `native`: the definition is rendered to a documented first-class product surface.
- `native-hook`: a documented lifecycle hook invokes Base, subject to the row's runtime/trust
  prerequisite.
- `partial-hook`: a native lifecycle binding exists, but the product does not cover the policy's
  complete declared event/tool domain.
- `hybrid-hook`: the hook mechanically blocks while approval is pending; denial-to-abort routing is
  still a behavioral pipeline contract.
- `assisted`: Base emits an executable instruction or behavioral fallback but cannot bind the full
  lifecycle contract.
- `advisory`: prose is the only available binding.

These are adapter-fidelity terms, not security certifications.

## Target profiles

- **Claude Code:** custom agents, project skills, and project hooks are native. Read-only access
  maps to subagent plan mode. The generated pre-tool hook inspects Bash/file/MCP events. A
  compatible `base` executable must be on `PATH`; `base check` can probe that prerequisite but
  cannot prove a separately launched environment will preserve it.
- **Codex:** subagents, Agent Skills, and `.codex/hooks.json` are native. Project hooks run only
  after the exact hook definition is trusted through `/hooks` (or deployed as managed policy).
  Base maps `session-start`, `pre-tool-use`, and `post-tool-use` directly. Canonical `session-end`
  stays `assisted` because Codex exposes turn-level `Stop`, not an equivalent session-end event.
  Codex does not invoke tool hooks for every hosted tool. Pre/post-tool policies are `native-hook`
  only when their `match-tools` entries are limited to `Bash`, `apply_patch`, or `mcp__*` names;
  an empty or broader matcher is reported as `partial-hook` and retains its behavioral fallback.
  `.codex/rules/base.rules` is a narrow defense-in-depth rule for common explicit refspecs, not the
  primary guard.
- **GitHub Copilot:** `.github/agents` custom agents, Agent Skills, and repository hooks are native
  to their documented product profiles. `.github/prompts` is a VS Code prompt-file surface; it is
  not presented as a Copilot CLI/cloud pipeline. Base uses PascalCase lifecycle events so payloads
  use Claude-compatible snake_case fields. The hook file applies to Copilot CLI/cloud; host hook
  timeouts are always fail-open. A live Copilot CLI 1.0.72 probe on 2026-07-20 exposed the built-in
  GitHub MCP namespace as `github-mcp-server-*`; Base maps canonical `mcp__github__*` globs to that
  namespace. Other canonical `mcp__<server>__*` globs are `partial-hook` for Copilot until an
  explicit server-name mapping exists.

Base compiles canonical `match-tools` full-name globs to explicitly anchored regular expressions.
It never passes raw host regex through, avoiding Claude's unanchored-regex behavior diverging from
Codex or Copilot matching. Mutation capabilities are translated to each host's aliases: canonical
`Edit`/`Write`/`NotebookEdit` become Codex `apply_patch`, while canonical `apply_patch` becomes
Claude/Copilot `Edit`; duplicates collapse deterministically. Copilot MCP server names are not a
portable mechanical transform; only the current built-in GitHub mapping is first-class in v0.2.

It is never reported as equivalent to `native-hook`.

Generated Copilot and Codex wrappers probe Base hook protocol 1. Built-in guards and canonical
`fail-closed: true` guards emit a target-native deny when Base is missing or incompatible;
context/observe and fail-open guards skip with no decision. The outer product may still fail open
on its own timeout. `base check` can prove the local binary probe, but not Codex trust state or a
remote Copilot cloud image.

The default-branch hook recognizes common `git`, `git.exe`, explicit/forced refspecs, `HEAD`/`@`
on the default branch, and GitHub MCP branch writes. Shell aliases, wrappers, disabled hooks, and
host-specific escape paths remain possible. Server-side branch protection is the authoritative
boundary.

Generated Markdown and TOML carry a visible do-not-edit marker. JSON and copied binary skill
resources cannot carry comments; the generated manifest still owns and hash-protects them.
Allowlisted `.base/native/` Markdown and JSON inputs are composed before hashing so established
project configuration remains reproducible without hand-editing generated output. Manifest and
drift hashes canonicalize CRLF to LF for valid UTF-8 content, while non-UTF-8 resources are compared
byte-for-byte; Base therefore coexists with a project's chosen Git attributes instead of rewriting
them.

## Verified references

- [Claude Code skills](https://code.claude.com/docs/en/skills)
- [Claude Code subagents](https://code.claude.com/docs/en/sub-agents)
- [Claude Code hooks](https://code.claude.com/docs/en/hooks)
- [Codex Agent Skills](https://learn.chatgpt.com/docs/build-skills)
- [Codex subagents](https://learn.chatgpt.com/docs/agent-configuration/subagents)
- [Codex hooks](https://learn.chatgpt.com/docs/hooks)
- [GitHub Copilot Agent Skills](https://docs.github.com/en/copilot/how-tos/copilot-on-github/customize-copilot/customize-cloud-agent/add-skills)
- [GitHub Copilot custom-agent configuration](https://docs.github.com/en/copilot/reference/custom-agents-configuration)
- [GitHub Copilot hooks](https://docs.github.com/en/copilot/reference/hooks-reference)
- [VS Code prompt files](https://code.visualstudio.com/docs/agent-customization/prompt-files)
