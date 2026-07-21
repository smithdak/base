use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use chrono::{SecondsFormat, Utc};
use serde::Serialize;

use crate::cli::ApproveArgs;
use crate::config::{Config, GateKind};

use super::{print_json, validate_run_slug};

#[derive(Debug, Serialize)]
struct ApprovalReport {
    run: String,
    gate: String,
    verdict: &'static str,
    by: String,
    at: String,
    path: String,
}

pub fn run(project_root: &Path, args: ApproveArgs, json: bool) -> Result<()> {
    validate_run_slug(&args.run)?;
    let config = Config::load(project_root)?;
    let Some(gate) = config.gate(&args.gate) else {
        bail!(
            "unknown gate `{}`; declare it in .base/base.toml",
            args.gate
        );
    };
    if gate.kind != GateKind::StageApproval {
        bail!("gate `{}` is not a stage-approval gate", args.gate);
    }

    let run_folder = project_root.join(".base/runs").join(&args.run);
    if !run_folder.is_dir() {
        bail!("no run folder at .base/runs/{}", args.run);
    }

    let request_relative = gate.request_path();
    let request = run_folder.join(&request_relative);
    if !request.is_file() {
        bail!(
            "gate `{}` has no pending request at .base/runs/{}/{}; verdicts cannot be recorded before the stage requests one",
            args.gate,
            args.run,
            request_relative
        );
    }

    let relative = gate.approval_path();
    let artifact = run_folder.join(&relative);

    let verdict = if args.deny { "denied" } else { "approved" };
    let by = args.by.clone().unwrap_or_else(|| decider(project_root));
    validate_field(&by, "approval decider")?;
    if let Some(note) = &args.note {
        validate_field(note, "approval note")?;
    }
    let at = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);

    let mut record = format!(
        "# Gate decision: {verdict}\n\n- gate: {}\n- run: {}\n- verdict: {verdict}\n- by: {by}\n- at: {at}\n",
        gate.id, args.run
    );
    if let Some(note) = &args.note {
        record.push_str(&format!("- note: {note}\n"));
    }

    if let Some(parent) = artifact.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("cannot create {}", parent.display()))?;
    }
    let mut file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&artifact)
    {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            bail!(
                "approval record already exists at .base/runs/{}/{relative}; verdicts are immutable — a changed decision belongs in a new run",
                args.run
            );
        }
        Err(error) => {
            return Err(error).with_context(|| format!("cannot reserve {}", artifact.display()));
        }
    };
    file.write_all(record.as_bytes())
        .with_context(|| format!("cannot write {}", artifact.display()))?;
    file.sync_all()
        .with_context(|| format!("cannot sync {}", artifact.display()))?;

    let report = ApprovalReport {
        path: format!(".base/runs/{}/{relative}", args.run),
        run: args.run,
        gate: gate.id.clone(),
        verdict,
        by,
        at,
    };
    if json {
        return print_json(&report);
    }
    println!(
        "recorded {} for gate `{}` on run {} at {}",
        report.verdict, report.gate, report.run, report.path
    );
    Ok(())
}

fn validate_field(value: &str, kind: &str) -> Result<()> {
    if value.trim().is_empty() || value.contains(['\r', '\n']) {
        bail!("{kind} must be one non-empty line");
    }
    Ok(())
}

fn decider(project_root: &Path) -> String {
    let git_name = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .args(["config", "user.name"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .filter(|name| !name.is_empty());
    git_name
        .or_else(|| std::env::var("USERNAME").ok())
        .or_else(|| std::env::var("USER").ok())
        .unwrap_or_else(|| "unknown".to_owned())
}
