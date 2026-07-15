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
    let command = event
        .pointer("/tool_input/command")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if pushes_default_branch(command, default_branch) {
        println!(
            "{}",
            json!({
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "deny",
                    "permissionDecisionReason": format!(
                        "base standing denial: never push directly to `{default_branch}`"
                    )
                }
            })
        );
    }
    Ok(())
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
    fn permits_explicit_feature_pushes() {
        assert!(!pushes_default_branch(
            "git push origin feature/base",
            "main"
        ));
        assert!(!pushes_default_branch("cargo test", "main"));
    }
}
