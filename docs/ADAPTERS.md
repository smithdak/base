# Adapter surfaces

Surface selection was verified on 2026-07-14. Adapters compile the same canonical pipeline prose;
only discovery metadata and the strongest available gate mechanism differ.

| Canon kind | Claude Code | Codex | GitHub Copilot |
|---|---|---|---|
| rules | `CLAUDE.md` | `AGENTS.md` | `.github/copilot-instructions.md` |
| pipeline | `.claude/skills/<id>/SKILL.md` | `.agents/skills/<id>/SKILL.md` | `.github/prompts/<id>.prompt.md` |
| agents | `.claude/agents/<id>.md` | advisory section in `AGENTS.md` | advisory section in instructions |
| standing denials | project permission rules + `PreToolUse` hook | project `.codex/rules/base.rules` | prose |
| knowledge | pointer list in `CLAUDE.md` | pointer list in `AGENTS.md` | pointer list in instructions |

Claude Code custom commands now share the skill model, while existing `.claude/commands/` remains
compatible. Codex repository-local custom prompts are deprecated and user-scoped, so repo pipelines
compile to the current cross-tool Agent Skills surface instead. Copilot prompt files remain a public
preview and are not available in every Copilot surface.

References:

- [Claude Code skills](https://code.claude.com/docs/en/skills)
- [Claude Code hooks](https://code.claude.com/docs/en/hooks)
- [Codex customization](https://developers.openai.com/codex/concepts/customization/)
- [GitHub Copilot repository instructions and prompt files](https://docs.github.com/en/copilot/how-tos/configure-custom-instructions-in-your-ide/add-repository-instructions-in-your-ide)

## Fidelity

`base check` reports every gate × active target as `enforced`, `assisted`, or `advisory`.

- Claude's standing default-branch denial is enforced for harness-issued Bash calls by a deny rule
  and a deterministic pre-tool hook. Stage approval remains assisted because the harness has no
  native stage boundary. The hook runs the `base` binary resolved from PATH, and Claude Code treats
  a command-not-found hook error as non-blocking — so `base check` probes PATH and reports the
  denial as `assisted` with a warning when the binary does not resolve, since only the deny rules
  (explicit refspecs) would actually fire.
- Codex forbids the ordinary explicit default-branch refspecs through project rules. Unusual or
  implicit refspecs and stage approval remain assisted.
- Copilot receives gate prose only, so both gate kinds are advisory.

Generated Markdown and Starlark files carry a visible do-not-edit marker. JSON cannot carry comments;
`.claude/settings.json` is protected by the same manifest hash without adding an unsupported setting.

