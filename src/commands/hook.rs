use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use command_group::CommandGroup;
use serde_json::{Value, json};

use crate::canon::{Policy, PolicyMode};
use crate::cli::{HookArgs, HookCommand};
use crate::config::{Config, GateKind, Target};
use crate::find_project_root;
use crate::process::{wait_group_timeout, wait_until};

use super::load_project;

const MAX_POLICY_OUTPUT: usize = 64 * 1024;
const MAX_HOOK_INPUT: u64 = 8 * 1024 * 1024;

pub fn run(args: HookArgs) -> Result<()> {
    match args.command {
        HookCommand::Capabilities {
            require,
            require_features,
        } => {
            let version = semver::Version::parse(env!("CARGO_PKG_VERSION"))
                .expect("Cargo package version is semantic");
            if let Some(requirement) = require {
                let requirement = semver::VersionReq::parse(&requirement)
                    .context("invalid required Base semantic-version range")?;
                if !requirement.matches(&version) {
                    bail!("Base {version} does not satisfy required range `{requirement}`");
                }
            }
            let features = ["pre-tool", "policy"];
            for feature in require_features {
                if !features.contains(&feature.as_str()) {
                    bail!("Base hook protocol 1 does not support required feature `{feature}`");
                }
            }
            println!(
                "{}",
                json!({
                    "protocol": 1,
                    "version": version.to_string(),
                    "features": features
                })
            );
            Ok(())
        }
        HookCommand::PreTool {
            target,
            default_branch,
        } => pre_tool(target, &default_branch),
        HookCommand::Policy { id, target } => run_policy(&id, target),
        HookCommand::ClaudePreTool { default_branch } => pre_tool(Target::Claude, &default_branch),
    }
}

fn pre_tool(target: Target, default_branch: &str) -> Result<()> {
    validate_hook_target(target)?;
    let input = read_input()?;
    let event: Value = serde_json::from_str(&input).context("invalid pre-tool hook JSON")?;
    let project_root = event_cwd(&event)
        .or_else(|| std::env::current_dir().ok())
        .and_then(|cwd| find_project_root(&cwd).ok());
    let current_branch = project_root.as_deref().and_then(current_branch);
    let denial = project_root
        .as_deref()
        .and_then(|root| approval_artifact_write_reason(&event, root))
        .or_else(|| denial_reason(&event, default_branch, current_branch.as_deref()))
        .or_else(|| {
            // Gate scan fails open: a filesystem oddity must not brick the session.
            // The standing denial above keeps its fail-closed posture.
            project_root.as_deref().and_then(pending_gate_reason)
        });
    if let Some(reason) = denial {
        emit_denial(target, &reason);
    }
    Ok(())
}

fn run_policy(id: &str, target: Target) -> Result<()> {
    validate_hook_target(target)?;
    let input = read_input()?;
    let event: Value = serde_json::from_str(&input).context("invalid lifecycle hook JSON")?;
    let start = event_cwd(&event)
        .or_else(|| std::env::current_dir().ok())
        .context("hook event has no usable working directory")?;
    let project_root = find_project_root(&start)?;
    let (_, canon) = load_project(&project_root)?;
    let policy = canon
        .policies
        .get(id)
        .with_context(|| format!("unknown policy `{id}`"))?;
    if !policy.source.repo_resident() {
        bail!("policy `{id}` is global-only and cannot back a repository hook");
    }

    let result = execute_policy(policy, &project_root, input.as_bytes());
    let output = match result {
        Ok(output) if output.timed_out => {
            return policy_failure(
                policy,
                target,
                format!(
                    "policy `{id}` timed out after {} seconds",
                    policy.meta.timeout_seconds
                ),
            );
        }
        Ok(output) if !output.status.success() => {
            let detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            let suffix = if detail.is_empty() {
                String::new()
            } else {
                format!(": {detail}")
            };
            return policy_failure(
                policy,
                target,
                format!("policy `{id}` exited {}{suffix}", output.status),
            );
        }
        Ok(output) => output,
        Err(error) => {
            return policy_failure(policy, target, format!("policy `{id}` failed: {error:#}"));
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    match policy.meta.mode {
        PolicyMode::Context => {
            if !stdout.is_empty() {
                emit_context(target, policy.meta.event.claude_name(), &stdout);
            }
        }
        PolicyMode::Observe => {}
        PolicyMode::Guard => {
            let decision: Value = match serde_json::from_str(&stdout) {
                Ok(decision) => decision,
                Err(error) => {
                    return policy_failure(
                        policy,
                        target,
                        format!("guard policy `{id}` must emit one JSON decision object: {error}"),
                    );
                }
            };
            match decision.get("decision").and_then(Value::as_str) {
                // Absence of an explicit decision is safer than granting around
                // another runtime permission or policy layer.
                Some("allow") => {}
                Some("deny") => {
                    let reason = decision
                        .get("reason")
                        .and_then(Value::as_str)
                        .filter(|reason| !reason.trim().is_empty())
                        .unwrap_or("denied by canonical Base policy");
                    emit_denial(target, reason);
                }
                value => {
                    return policy_failure(
                        policy,
                        target,
                        format!(
                            "guard policy `{id}` emitted unsupported decision `{}`",
                            value.unwrap_or("missing")
                        ),
                    );
                }
            }
        }
    }
    Ok(())
}

fn policy_failure(policy: &Policy, target: Target, reason: String) -> Result<()> {
    if policy.meta.mode == PolicyMode::Guard && policy.meta.fail_closed {
        emit_denial(target, &reason);
    } else {
        eprintln!("base hook warning: {reason}");
    }
    Ok(())
}

struct PolicyOutput {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    timed_out: bool,
}

fn execute_policy(policy: &Policy, project_root: &Path, input: &[u8]) -> Result<PolicyOutput> {
    let (program, args) = policy
        .meta
        .command
        .split_first()
        .expect("canon validation requires policy argv");
    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(project_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .group_spawn()
        .with_context(|| format!("cannot start policy command `{program}`"))?;

    let mut stdin = child
        .inner()
        .stdin
        .take()
        .context("policy stdin unavailable")?;
    let input = input.to_vec();
    let stdin_writer = thread::spawn(move || {
        let result = stdin.write_all(&input);
        drop(stdin);
        result
    });

    let stdout = child
        .inner()
        .stdout
        .take()
        .context("policy stdout unavailable")?;
    let stderr = child
        .inner()
        .stderr
        .take()
        .context("policy stderr unavailable")?;
    let stdout_reader = thread::spawn(move || read_capped(stdout, MAX_POLICY_OUTPUT));
    let stderr_reader = thread::spawn(move || read_capped(stderr, MAX_POLICY_OUTPUT));

    let timeout = Duration::from_secs(policy.meta.timeout_seconds);
    let waited = wait_group_timeout(&mut child, timeout, || {
        stdin_writer.is_finished() && stdout_reader.is_finished() && stderr_reader.is_finished()
    });
    let status = match waited {
        Ok(Some(status)) => (status, false),
        Ok(None) => {
            let _ = child.kill();
            let status = child.wait().context("cannot reap timed-out policy")?;
            wait_until(Duration::from_secs(1), || {
                stdin_writer.is_finished()
                    && stdout_reader.is_finished()
                    && stderr_reader.is_finished()
            });
            (status, true)
        }
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            wait_until(Duration::from_secs(1), || {
                stdin_writer.is_finished()
                    && stdout_reader.is_finished()
                    && stderr_reader.is_finished()
            });
            return Err(error).context("cannot wait for policy");
        }
    };
    let stdout = join_policy_reader(stdout_reader, "stdout")?;
    let stderr = join_policy_reader(stderr_reader, "stderr")?;
    // A child that exits without consuming stdin may close the pipe early; the
    // process status and policy protocol remain authoritative. Joining here is
    // essential so a timed-out non-reader cannot leave the writer behind.
    if stdin_writer.is_finished() {
        let _ = stdin_writer
            .join()
            .map_err(|_| anyhow::anyhow!("policy stdin writer panicked"))?;
    }
    Ok(PolicyOutput {
        status: status.0,
        stdout,
        stderr,
        timed_out: status.1,
    })
}

fn join_policy_reader(
    reader: thread::JoinHandle<io::Result<Vec<u8>>>,
    stream: &str,
) -> Result<Vec<u8>> {
    if !reader.is_finished() {
        // Cleanup has already exceeded its bounded grace period. Detach the
        // reader rather than turning a policy timeout into an unbounded hang.
        return Ok(Vec::new());
    }
    reader
        .join()
        .map_err(|_| anyhow::anyhow!("policy {stream} reader panicked"))?
        .with_context(|| format!("cannot read policy {stream}"))
}

fn read_capped(mut reader: impl Read, limit: usize) -> io::Result<Vec<u8>> {
    let mut retained = Vec::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        let remaining = limit.saturating_sub(retained.len());
        retained.extend_from_slice(&buffer[..read.min(remaining)]);
    }
    Ok(retained)
}

fn read_input() -> Result<String> {
    let mut input = String::new();
    io::stdin()
        .take(MAX_HOOK_INPUT + 1)
        .read_to_string(&mut input)?;
    if input.len() as u64 > MAX_HOOK_INPUT {
        bail!(
            "hook input exceeds the {} byte safety limit",
            MAX_HOOK_INPUT
        );
    }
    Ok(input)
}

fn event_cwd(event: &Value) -> Option<PathBuf> {
    event
        .get("cwd")
        .and_then(Value::as_str)
        .filter(|cwd| !cwd.trim().is_empty())
        .map(PathBuf::from)
}

fn validate_hook_target(target: Target) -> Result<()> {
    let _ = target;
    Ok(())
}

fn emit_denial(target: Target, reason: &str) {
    let output = match target {
        Target::Claude | Target::Codex => json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": reason
            }
        }),
        Target::Copilot => json!({
            "permissionDecision": "deny",
            "permissionDecisionReason": reason
        }),
    };
    println!("{output}");
}

fn emit_context(target: Target, event: &str, context: &str) {
    let output = match target {
        Target::Claude | Target::Codex => json!({
            "hookSpecificOutput": {
                "hookEventName": event,
                "additionalContext": context
            }
        }),
        Target::Copilot => json!({ "additionalContext": context }),
    };
    println!("{output}");
}

/// A request is pending until an explicit verdict exists. Approval resumes the
/// planned path; denial terminates that path and allows only the pipeline's
/// aborted-result recording. Malformed verdicts remain fail-closed.
fn pending_gate_reason(project_root: &Path) -> Option<String> {
    let runs = project_root.join(".base/runs");
    if !runs.is_dir() {
        return None;
    }
    let config = Config::load(project_root).ok()?;
    let request_paths: Vec<(String, String)> = config
        .gates
        .iter()
        .filter(|gate| gate.kind == GateKind::StageApproval && gate.satisfied_by.is_some())
        .map(|gate| (gate.id.clone(), gate.request_path()))
        .collect();
    let mut run_directories: Vec<PathBuf> = fs::read_dir(&runs)
        .ok()?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect();
    run_directories.sort();
    for run in run_directories {
        for (gate_id, request_path) in &request_paths {
            let path = run.join(request_path.replace('/', std::path::MAIN_SEPARATOR_STR));
            if !path.is_file() {
                continue;
            }
            let response = run.join(
                request_path
                    .strip_suffix(".request")
                    .expect("request paths always have a request suffix")
                    .replace('/', std::path::MAIN_SEPARATOR_STR),
            );
            let pending = path
                .strip_prefix(project_root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let run_and_gate = run_and_gate(&runs, &path, gate_id);
            if response.is_file() {
                let verdict = fs::read_to_string(&response).ok().and_then(|source| {
                    source.lines().find_map(|line| {
                        line.trim()
                            .strip_prefix("- verdict:")
                            .map(str::trim)
                            .map(str::to_owned)
                    })
                });
                match verdict.as_deref() {
                    Some("approved") => continue,
                    Some("denied") => continue,
                    _ => {
                        return Some(format!(
                            "base stage gate: {run_and_gate} has a malformed verdict artifact; mutation remains denied"
                        ));
                    }
                }
            }
            return Some(format!(
                "base stage gate: {run_and_gate} awaits a human verdict ({pending} has no response). Record one from your own terminal: `base approve {}` or `base approve {} --deny`. Mutating tools stay denied until the verdict artifact exists.",
                run_and_gate, run_and_gate
            ));
        }
    }
    None
}

fn run_and_gate(runs: &Path, request: &Path, gate_id: &str) -> String {
    let slug = request
        .strip_prefix(runs)
        .ok()
        .and_then(|rel| rel.components().next())
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .unwrap_or_default();
    format!("{slug} {gate_id}")
}

fn approval_artifact_write_reason(event: &Value, project_root: &Path) -> Option<String> {
    let config = Config::load(project_root).ok()?;
    let approvals: Vec<String> = config
        .gates
        .iter()
        .filter(|gate| gate.kind == GateKind::StageApproval && gate.satisfied_by.is_some())
        .map(crate::config::Gate::approval_path)
        .collect();
    if approvals.is_empty() {
        return None;
    }
    let tool = event
        .get("tool_name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let input = event.get("tool_input").unwrap_or(&Value::Null);
    let patch_targeted = input
        .as_str()
        .or_else(|| input.get("patch").and_then(Value::as_str))
        .or_else(|| input.get("input").and_then(Value::as_str))
        .is_some_and(|patch| patch_targets_approval(patch, &approvals));
    let targeted = if tool == "apply_patch" {
        patch_targeted
    } else if matches!(
        tool.as_str(),
        "write" | "edit" | "notebookedit" | "notebook_edit"
    ) || github_branch_write_tool(&tool)
    {
        patch_targeted || path_fields_target_approval(input, None, &approvals)
    } else if tool == "bash" || tool == "shell" || tool == "shell_command" {
        input
            .get("command")
            .and_then(Value::as_str)
            .is_some_and(|command| shell_mutates_approval(command, &approvals))
    } else {
        false
    };
    targeted.then(|| {
        "base stage gate: approval verdict artifacts are human-owned and cannot be written or changed by an agent tool; use `base approve` from your own terminal"
            .to_owned()
    })
}

fn patch_targets_approval(patch: &str, approvals: &[String]) -> bool {
    patch.lines().any(|line| {
        ["*** Add File: ", "*** Update File: ", "*** Delete File: "]
            .iter()
            .find_map(|prefix| line.trim().strip_prefix(prefix))
            .is_some_and(|path| path_targets_approval(path, approvals))
    })
}

fn path_fields_target_approval(value: &Value, field: Option<&str>, approvals: &[String]) -> bool {
    match value {
        Value::Object(object) => object.iter().any(|(key, value)| {
            path_fields_target_approval(value, Some(&key.to_ascii_lowercase()), approvals)
        }),
        Value::Array(values) => values
            .iter()
            .any(|value| path_fields_target_approval(value, field, approvals)),
        Value::String(path)
            if field.is_some_and(|field| {
                field.contains("path") || field == "file" || field == "filename"
            }) =>
        {
            path_targets_approval(path, approvals)
        }
        _ => false,
    }
}

fn path_targets_approval(path: &str, approvals: &[String]) -> bool {
    let normalized = path.replace('\\', "/").to_ascii_lowercase();
    approvals.iter().any(|approval| {
        let approval = approval.to_ascii_lowercase();
        normalized == approval || normalized.ends_with(&format!("/{approval}"))
    })
}

fn shell_mutates_approval(command: &str, approvals: &[String]) -> bool {
    let normalized = command.replace('\\', "/").to_ascii_lowercase();
    let mut without_requests = normalized.clone();
    for approval in approvals {
        let request = format!("{}.request", approval.to_ascii_lowercase());
        without_requests = without_requests.replace(&request, "<base-request>");
    }
    let names_path = approvals.iter().any(|approval| {
        let approval = approval.to_ascii_lowercase();
        without_requests.contains(&approval)
    });
    let mutation = [
        "set-content",
        "out-file",
        "new-item",
        "remove-item",
        "move-item",
        "copy-item",
        "apply_patch",
        " tee ",
        " sed ",
        " rm ",
        " mv ",
        " cp ",
        ">",
    ]
    .iter()
    .any(|marker| normalized.contains(marker));
    names_path && mutation
}

fn denial_reason(
    event: &Value,
    default_branch: &str,
    current_branch: Option<&str>,
) -> Option<String> {
    let tool_name = event
        .pointer("/tool_name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if github_branch_write_tool(tool_name) {
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
    if invokes_base_approve(command) {
        return Some(
            "base stage gate: `base approve` must be run by the human decider from a terminal outside the agent session"
                .to_owned(),
        );
    }
    if pushes_default_branch(command, default_branch, current_branch) {
        return Some(format!(
            "base standing denial: never push directly to `{default_branch}`"
        ));
    }
    None
}

fn github_branch_write_tool(tool_name: &str) -> bool {
    let tool_name = tool_name.to_ascii_lowercase();
    if !tool_name.starts_with("mcp__github__") && !tool_name.starts_with("github-mcp-server-") {
        return false;
    }
    [
        "push_files",
        "create_or_update_file",
        "delete_file",
        "update_file",
    ]
    .iter()
    .any(|operation| tool_name.ends_with(operation))
}

fn invokes_base_approve(command: &str) -> bool {
    // Normalize Windows separators before POSIX-style tokenization; otherwise
    // shell_words treats each backslash as an escape and hides `base.exe`.
    let normalized = command.replace('\\', "/").replace([';', '|', '&'], " ; ");
    let Ok(tokens) = shell_words::split(&normalized) else {
        return false;
    };
    tokens.windows(2).any(|pair| {
        let executable = Path::new(&pair[0])
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or(&pair[0]);
        executable.eq_ignore_ascii_case("base") && pair[1] == "approve"
    })
}

fn pushes_default_branch(
    command: &str,
    default_branch: &str,
    current_branch: Option<&str>,
) -> bool {
    let normalized = command.replace('\\', "/").replace([';', '|', '&'], " ; ");
    match shell_words::split(&normalized) {
        Ok(tokens) => tokens.split(|token| token == ";").any(|command_tokens| {
            command_pushes_default_branch(command_tokens, default_branch, current_branch)
        }),
        Err(_) => {
            let words: Vec<&str> = command
                .split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '-' || c == '/'))
                .filter(|word| !word.is_empty())
                .collect();
            words.contains(&"git")
                && words.contains(&"push")
                && words.iter().any(|word| {
                    *word == default_branch || word.ends_with(&format!("/{default_branch}"))
                })
        }
    }
}

fn command_pushes_default_branch(
    tokens: &[String],
    default_branch: &str,
    current_branch: Option<&str>,
) -> bool {
    let Some(git_index) = tokens.iter().position(|token| {
        Path::new(token)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .is_some_and(|stem| stem.eq_ignore_ascii_case("git"))
    }) else {
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
        if arguments
            .iter()
            .any(|argument| matches!(argument.as_str(), "--all" | "--mirror"))
        {
            return true;
        }
        return current_branch.is_none_or(|branch| branch == default_branch);
    }
    non_flags[1..].iter().any(|refspec| {
        let refspec = refspec.trim_start_matches('+');
        ((refspec == "HEAD" || refspec == "@") && current_branch == Some(default_branch))
            || refspec == default_branch
            || refspec.ends_with(&format!(":{default_branch}"))
            || refspec.ends_with(&format!("/heads/{default_branch}"))
    })
}

fn current_branch(project_root: &Path) -> Option<String> {
    Command::new("git")
        .arg("-C")
        .arg(project_root)
        .args(["symbolic-ref", "--quiet", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .filter(|branch| !branch.is_empty())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn pending_request_denies_until_answered() {
        let project = tempfile::TempDir::new().unwrap();
        fs::create_dir_all(project.path().join(".base")).unwrap();
        Config::default().save(project.path()).unwrap();
        let run = project.path().join(".base/runs/2026-07-15-demo/approvals");
        fs::create_dir_all(&run).unwrap();
        assert!(pending_gate_reason(project.path()).is_none());
        fs::write(run.join("plan-approval.md.request"), "what needs approval").unwrap();
        let reason = pending_gate_reason(project.path()).expect("pending gate denies");
        assert!(reason.contains("2026-07-15-demo plan-approval"), "{reason}");
        assert!(reason.contains("base approve"), "{reason}");
        fs::write(
            run.join("plan-approval.md"),
            "# Gate decision: approved\n\n- verdict: approved\n",
        )
        .unwrap();
        assert!(pending_gate_reason(project.path()).is_none());

        fs::remove_file(run.join("plan-approval.md")).unwrap();
        fs::write(
            run.join("plan-approval.md"),
            "# Gate decision: denied\n\n- verdict: denied\n",
        )
        .unwrap();
        assert!(pending_gate_reason(project.path()).is_none());
    }

    #[test]
    fn projects_without_runs_do_not_scan() {
        let project = tempfile::TempDir::new().unwrap();
        assert!(pending_gate_reason(project.path()).is_none());
    }

    #[test]
    fn unrelated_or_removed_request_files_do_not_gate_mutation() {
        let project = tempfile::TempDir::new().unwrap();
        fs::create_dir_all(project.path().join(".base")).unwrap();
        Config::default().save(project.path()).unwrap();
        let run = project.path().join(".base/runs/demo");
        fs::create_dir_all(run.join("evidence")).unwrap();
        fs::create_dir_all(run.join("approvals")).unwrap();
        fs::write(run.join("evidence/verifier.request"), "fixture").unwrap();
        fs::write(run.join("approvals/removed-gate.md.request"), "stale").unwrap();
        assert!(pending_gate_reason(project.path()).is_none());
    }

    #[test]
    fn custom_approval_paths_still_report_the_configured_gate_id() {
        let project = tempfile::TempDir::new().unwrap();
        fs::create_dir_all(project.path().join(".base")).unwrap();
        let mut config = Config::default();
        config.gates[0].id = "security-review".to_owned();
        config.gates[0].satisfied_by = Some("approvals/human-verdict.md".to_owned());
        config.save(project.path()).unwrap();
        let run = project.path().join(".base/runs/demo/approvals");
        fs::create_dir_all(&run).unwrap();
        fs::write(run.join("human-verdict.md.request"), "pending").unwrap();

        let reason = pending_gate_reason(project.path()).unwrap();
        assert!(
            reason.contains("base approve demo security-review"),
            "{reason}"
        );
        assert!(
            !reason.contains("base approve demo human-verdict"),
            "{reason}"
        );
    }

    #[test]
    fn agent_tools_cannot_create_or_rewrite_human_verdict_artifacts() {
        let project = tempfile::TempDir::new().unwrap();
        fs::create_dir_all(project.path().join(".base")).unwrap();
        Config::default().save(project.path()).unwrap();
        let verdict = project
            .path()
            .join(".base/runs/demo/approvals/plan-approval.md")
            .display()
            .to_string();
        let write = json!({
            "tool_name": "Write",
            "tool_input": {"file_path": verdict, "content": "- verdict: approved"}
        });
        assert!(approval_artifact_write_reason(&write, project.path()).is_some());
        let patch = json!({
            "tool_name": "apply_patch",
            "tool_input": {"patch": "*** Begin Patch\n*** Add File: .base/runs/demo/approvals/plan-approval.md\n+- verdict: approved\n*** End Patch"}
        });
        assert!(approval_artifact_write_reason(&patch, project.path()).is_some());
        let copilot_alias = json!({
            "tool_name": "Edit",
            "tool_input": {"patch": "*** Begin Patch\n*** Add File: .base/runs/demo/approvals/plan-approval.md\n+- verdict: approved\n*** End Patch"}
        });
        assert!(approval_artifact_write_reason(&copilot_alias, project.path()).is_some());
        let request = json!({
            "tool_name": "apply_patch",
            "tool_input": {"patch": "*** Begin Patch\n*** Add File: .base/runs/demo/approvals/plan-approval.md.request\n+Please review.\n*** End Patch"}
        });
        assert!(approval_artifact_write_reason(&request, project.path()).is_none());
        let read = json!({
            "tool_name": "Bash",
            "tool_input": {"command": "Get-Content .base/runs/demo/approvals/plan-approval.md"}
        });
        assert!(approval_artifact_write_reason(&read, project.path()).is_none());
        let shell_write = json!({
            "tool_name": "Bash",
            "tool_input": {"command": "Set-Content .base/runs/demo/approvals/plan-approval.md approved"}
        });
        assert!(approval_artifact_write_reason(&shell_write, project.path()).is_some());
        let mcp_read = json!({
            "tool_name": "mcp__github__get_file_contents",
            "tool_input": {"branch": "main", "path": ".base/runs/demo/approvals/plan-approval.md"}
        });
        assert!(approval_artifact_write_reason(&mcp_read, project.path()).is_none());
        let mcp_write = json!({
            "tool_name": "mcp__github__create_or_update_file",
            "tool_input": {"branch": "feature", "path": ".base/runs/demo/approvals/plan-approval.md"}
        });
        assert!(approval_artifact_write_reason(&mcp_write, project.path()).is_some());
    }

    #[test]
    fn catches_explicit_and_implicit_default_pushes() {
        assert!(pushes_default_branch("git push origin main", "main", None));
        assert!(pushes_default_branch(
            "git push origin HEAD:main",
            "main",
            None
        ));
        assert!(pushes_default_branch("git push", "main", None));
        assert!(pushes_default_branch(
            "npm test && git push origin main",
            "main",
            None
        ));
        assert!(pushes_default_branch("git push origin +main", "main", None));
        assert!(pushes_default_branch(
            "git push origin +HEAD:main",
            "main",
            None
        ));
        assert!(pushes_default_branch(
            r#""C:\Program Files\Git\cmd\git.exe" push origin main"#,
            "main",
            None
        ));
        assert!(pushes_default_branch(
            "git push origin HEAD",
            "main",
            Some("main")
        ));
        assert!(!pushes_default_branch(
            "git push origin HEAD",
            "main",
            Some("feature/x")
        ));
        assert!(!pushes_default_branch(
            "git push",
            "main",
            Some("feature/x")
        ));
        assert!(pushes_default_branch(
            "git push --all",
            "main",
            Some("feature/x")
        ));
    }

    #[test]
    fn blocks_agent_invocation_of_the_human_approval_command() {
        assert!(invokes_base_approve("base approve run plan-approval"));
        assert!(invokes_base_approve(
            r#"C:\Users\tester\.cargo\bin\base.exe approve run plan-approval"#
        ));
        assert!(!invokes_base_approve(
            r#"git commit -m "document base approve syntax""#
        ));
    }

    #[test]
    fn denies_github_mcp_writes_to_default_branch() {
        let push = serde_json::json!({
            "tool_name": "mcp__github__push_files",
            "tool_input": {"owner": "o", "repo": "r", "branch": "main", "message": "m"}
        });
        assert!(denial_reason(&push, "main", None).is_some());
        let update = serde_json::json!({
            "tool_name": "mcp__github__create_or_update_file",
            "tool_input": {"branch": "refs/heads/main"}
        });
        assert!(denial_reason(&update, "main", None).is_some());
        let copilot_live_shape = serde_json::json!({
            "tool_name": "github-mcp-server-push_files",
            "tool_input": {"owner": "o", "repo": "r", "branch": "main", "message": "m"}
        });
        assert!(denial_reason(&copilot_live_shape, "main", None).is_some());
        let unrelated_suffix = serde_json::json!({
            "tool_name": "custom-update_file",
            "tool_input": {"branch": "main"}
        });
        assert!(denial_reason(&unrelated_suffix, "main", None).is_none());
    }

    #[test]
    fn permits_github_mcp_review_path() {
        let feature = serde_json::json!({
            "tool_name": "mcp__github__push_files",
            "tool_input": {"branch": "feature/x"}
        });
        assert!(denial_reason(&feature, "main", None).is_none());
        let merge = serde_json::json!({
            "tool_name": "mcp__github__merge_pull_request",
            "tool_input": {"pullNumber": 2}
        });
        assert!(denial_reason(&merge, "main", None).is_none());
    }

    #[test]
    fn permits_github_mcp_reads_from_the_default_branch() {
        let read = serde_json::json!({
            "tool_name": "mcp__github__get_file_contents",
            "tool_input": {"branch": "main", "path": "README.md"}
        });
        assert!(denial_reason(&read, "main", None).is_none());
        let copilot_read = serde_json::json!({
            "tool_name": "github-mcp-server-get_file_contents",
            "tool_input": {"branch": "main", "path": "README.md"}
        });
        assert!(denial_reason(&copilot_read, "main", None).is_none());
    }

    #[test]
    fn permits_explicit_feature_pushes() {
        assert!(!pushes_default_branch(
            "git push origin feature/base",
            "main",
            None
        ));
        assert!(!pushes_default_branch("cargo test", "main", None));
    }

    #[test]
    fn permits_quoted_text_mentioning_pushes() {
        assert!(!pushes_default_branch(
            r#"git commit -m "x; git push remains""#,
            "main",
            None
        ));
        assert!(!pushes_default_branch(
            "git commit -q -m \"$(cat <<'EOF'\nbound only to Bash git push;\nchanges remain the review path\nEOF\n)\" && git push -u origin feat/w-0002-mcp-gate",
            "main",
            None
        ));
    }

    #[test]
    fn catches_no_space_separators() {
        assert!(pushes_default_branch(
            "cd x&&git push origin main",
            "main",
            None
        ));
        assert!(pushes_default_branch("true||git push", "main", None));
    }

    #[test]
    fn unparseable_input_fails_closed_on_words() {
        assert!(pushes_default_branch(
            "git push origin main \"oops",
            "main",
            None
        ));
        assert!(!pushes_default_branch(
            "echo \"unbalanced git push remains",
            "main",
            None
        ));
    }
}
