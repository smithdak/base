$ base check
canon valid: 1 rules, 2 agents, 6 stages, 2 pipelines, 2 knowledge entries

GATE                          KIND              TARGET    FIDELITY
plan-approval                 stage-approval    claude    assisted
plan-approval                 stage-approval    codex     assisted
plan-approval                 stage-approval    copilot   advisory
never-push-default-branch     standing-denial   claude    enforced
never-push-default-branch     standing-denial   codex     assisted
never-push-default-branch     standing-denial   copilot   advisory

claude: standing denial uses a project permission deny plus a PreToolUse hook; stage approval is prompt-assisted
codex: explicit default-branch refspecs are blocked by project rules; stage approval and unusual refspecs remain assisted
copilot: gates are declared in prose and remain advisory

$ base sync
synced 13 files (9 written, 4 unchanged, 0 removed)

$ base sync --check
sync check passed (13 generated files)

$ grep -n "^## " .claude/skills/build/SKILL.md
18:## 1. Intake
24:## 2. Plan
33:## 3. Execute
39:## 4. Verify
46:## 5. Record

$ ls fix surfaces
.agents/skills/fix/SKILL.md
.claude/skills/fix/SKILL.md
.github/prompts/fix.prompt.md

$ git clone -b feat/fix-pipeline-verify-stage <repo> fresh && BASE_HOME=<nonexistent> base --directory fresh sync --check
sync check passed (13 generated files)
