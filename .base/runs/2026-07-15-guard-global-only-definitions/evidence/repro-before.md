step 1: sync clone against global canon holding global-only rule/agent/stage/pipeline
synced 17 files (7 written, 10 unchanged, 0 removed)
step 2: global bytes now in committed surfaces:
11:Never force-push shared branches.
28:- **researcher** — Read-only exploration agent.
35:- `/ship` — Deploy and record
/c/Users/dakot/AppData/Local/Temp/claude/D--github-base/02acc0a0-77ed-4f3e-bc79-4dd02b6c93ff/scratchpad/clone-w5/.claude/agents/:
builder.md
researcher.md
reviewer.md

/c/Users/dakot/AppData/Local/Temp/claude/D--github-base/02acc0a0-77ed-4f3e-bc79-4dd02b6c93ff/scratchpad/clone-w5/.claude/skills/:
build
fix
ship
step 3: same repo without that global canon (fresh machine / CI):
error: generated output is out of sync: content differs .github/copilot-instructions.md, content differs AGENTS.md, content differs CLAUDE.md, stale manifest entry .agents/skills/ship/SKILL.md, stale manifest entry .claude/agents/researcher.md, stale manifest entry .claude/skills/ship/SKILL.md, stale manifest entry .github/prompts/ship.prompt.md
exit: 1
