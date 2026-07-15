use std::io::{self, Read};

use anyhow::{Context, Result};
use serde_json::{Value, json};

use crate::cli::{HookArgs, HookCommand};

pub fn run(args: HookArgs) -> Result<()> {
    match args.command {
        HookCommand::ClaudePreTool { default_branch } => claude_pre_tool(&default_branch),
    }
}

fn claude_pre_tool(default_branch: &str) -> Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let event: Value = serde_json::from_str(&input).context("invalid Claude hook JSON")?;
    if let Some(reason) = denial_reason(&event, default_branch) {
        println!(
            "{}",
            json!({
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "deny",
                    "permissionDecisionReason": reason
                }
            })
        );
    }
    Ok(())
}

fn denial_reason(event: &Value, default_branch: &str) -> Option<String> {
    let tool_name = event
        .pointer("/tool_name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if tool_name.starts_with("mcp__github__") {
        // Direct-write tools carry the target in `branch`; review-path tools
        // (create_pull_request, merge_pull_request) carry no branch to write to.
        let branch = event
            .pointer("/tool_input/branch")
            .and_then(Value::as_str)?;
        let branch = branch.strip_prefix("refs/heads/").unwrap_or(branch);
        if branch == default_branch {
            return Some(format!(
                "base standing denial: never write to `{default_branch}` via GitHub MCP; push a feature branch and open a review instead"
            ));
        }
        return None;
    }
    let command = event
        .pointer("/tool_input/command")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if pushes_default_branch(command, default_branch) {
        return Some(format!(
            "base standing denial: never push directly to `{default_branch}`"
        ));
    }
    None
}

fn pushes_default_branch(command: &str, default_branch: &str) -> bool {
    let normalized = command
        .replace("&&", " ; ")
        .replace("||", " ; ")
        .replace('|', " ; ");
    normalized.split(';').any(|part| {
        let Ok(tokens) = shell_words::split(part) else {
            return part.contains("git") && part.contains("push") && part.contains(default_branch);
        };
        let Some(git_index) = tokens.iter().position(|token| token == "git") else {
            return false;
        };
        let Some(push_offset) = tokens[git_index + 1..]
            .iter()
            .position(|token| token == "push")
        else {
            return false;
        };
        let arguments = &tokens[git_index + push_offset + 2..];
        let non_flags: Vec<&str> = arguments
            .iter()
            .map(String::as_str)
            .filter(|token| !token.starts_with('-'))
            .collect();

        if non_flags.len() < 2 {
            // An implicit push can target the checked-out default branch. Fail closed.
            return true;
        }
        non_flags[1..].iter().any(|refspec| {
            *refspec == default_branch
                || refspec.ends_with(&format!(":{default_branch}"))
                || refspec.ends_with(&format!("/heads/{default_branch}"))
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catches_explicit_and_implicit_default_pushes() {
        assert!(pushes_default_branch("git push origin main", "main"));
        assert!(pushes_default_branch("git push origin HEAD:main", "main"));
        assert!(pushes_default_branch("git push", "main"));
        assert!(pushes_default_branch(
            "npm test && git push origin main",
            "main"
        ));
    }

    #[test]
    fn denies_github_mcp_writes_to_default_branch() {
        let push = serde_json::json!({
            "tool_name": "mcp__github__push_files",
            "tool_input": {"owner": "o", "repo": "r", "branch": "main", "message": "m"}
        });
        assert!(denial_reason(&push, "main").is_some());
        let update = serde_json::json!({
            "tool_name": "mcp__github__create_or_update_file",
            "tool_input": {"branch": "refs/heads/main"}
        });
        assert!(denial_reason(&update, "main").is_some());
    }

    #[test]
    fn permits_github_mcp_review_path() {
        let feature = serde_json::json!({
            "tool_name": "mcp__github__push_files",
            "tool_input": {"branch": "feature/x"}
        });
        assert!(denial_reason(&feature, "main").is_none());
        let merge = serde_json::json!({
            "tool_name": "mcp__github__merge_pull_request",
            "tool_input": {"pullNumber": 2}
        });
        assert!(denial_reason(&merge, "main").is_none());
        let bash = serde_json::json!({
            "tool_name": "Bash",
            "tool_input": {"command": "git push origin main"}
        });
        assert!(denial_reason(&bash, "main").is_some());
    }

    #[test]
    fn permits_explicit_feature_pushes() {
        assert!(!pushes_default_branch(
            "git push origin feature/base",
            "main"
        ));
        assert!(!pushes_default_branch("cargo test", "main"));
    }
}
