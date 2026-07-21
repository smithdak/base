use std::fmt::Write as _;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use chrono::{SecondsFormat, Utc};
use command_group::CommandGroup;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::canon::{Layer, VerifierCheck};
use crate::cli::VerifyArgs;
use crate::process::{wait_group_timeout, wait_until};

use super::{load_project, print_json, validate_run_slug};

const MAX_CAPTURE_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Verdict {
    Pass,
    Fail,
    Inconclusive,
}

#[derive(Debug, Serialize)]
struct VerifyReport {
    suite: String,
    description: String,
    verdict: Verdict,
    started_at: String,
    finished_at: String,
    checks: Vec<CheckReport>,
    evidence_path: Option<String>,
}

#[derive(Debug, Serialize)]
struct CheckReport {
    id: String,
    argv: Vec<String>,
    cwd: String,
    verdict: Verdict,
    exit_code: Option<i32>,
    duration_ms: i64,
    stdout_bytes: u64,
    stderr_bytes: u64,
    stdout_sha256: String,
    stderr_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stderr: Option<String>,
    output_retained: bool,
    output_truncated: bool,
    note: Option<String>,
}

pub fn run(project_root: &Path, args: VerifyArgs, json: bool) -> Result<()> {
    let (_, canon) = load_project(project_root)?;
    let verifier = canon
        .verifiers
        .get(&args.suite)
        .with_context(|| format!("unknown verifier suite `{}`", args.suite))?;
    if verifier.source.layer == Layer::Global {
        bail!(
            "verifier `{}` is global-only; adopt it into the repository before execution",
            args.suite
        );
    }

    let started = Utc::now();
    let evidence = args
        .run
        .as_deref()
        .map(|slug| reserve_evidence(project_root, slug, &args.suite, started))
        .transpose()?;
    let mut checks = Vec::new();
    for check in &verifier.meta.checks {
        checks.push(run_check(project_root, check));
    }
    let verdict = if checks.iter().any(|check| check.verdict == Verdict::Fail) {
        Verdict::Fail
    } else if checks
        .iter()
        .any(|check| check.verdict == Verdict::Inconclusive)
    {
        Verdict::Inconclusive
    } else {
        Verdict::Pass
    };
    let finished = Utc::now();
    let report = VerifyReport {
        suite: args.suite,
        description: verifier.meta.description.clone(),
        verdict,
        started_at: started.to_rfc3339_opts(SecondsFormat::Millis, true),
        finished_at: finished.to_rfc3339_opts(SecondsFormat::Millis, true),
        checks,
        evidence_path: evidence.as_ref().map(|item| item.relative.clone()),
    };

    if let Some(mut evidence) = evidence {
        let source = serde_json::to_vec_pretty(&report)?;
        evidence
            .file
            .write_all(&source)
            .with_context(|| format!("cannot write {}", evidence.path.display()))?;
        evidence
            .file
            .sync_all()
            .with_context(|| format!("cannot sync {}", evidence.path.display()))?;
    }

    if json {
        print_json(&report)?;
    } else {
        println!("verifier `{}`: {:?}", report.suite, report.verdict);
        for check in &report.checks {
            println!(
                "  {:<24} {:<12} {} ms",
                check.id,
                format!("{:?}", check.verdict).to_lowercase(),
                check.duration_ms
            );
            if let Some(note) = &check.note {
                println!("    {note}");
            }
        }
        if let Some(path) = &report.evidence_path {
            println!("evidence: {path}");
        }
    }
    match report.verdict {
        Verdict::Pass => Ok(()),
        Verdict::Fail => bail!("verifier `{}` failed", report.suite),
        Verdict::Inconclusive => bail!("verifier `{}` was inconclusive", report.suite),
    }
}

fn run_check(project_root: &Path, check: &VerifierCheck) -> CheckReport {
    let started = Utc::now();
    let cwd_path = check
        .cwd
        .as_deref()
        .map(|relative| project_root.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR)))
        .unwrap_or_else(|| project_root.to_path_buf());
    let cwd = cwd_path
        .strip_prefix(project_root)
        .unwrap_or(&cwd_path)
        .to_string_lossy()
        .replace('\\', "/");
    let cwd = if cwd.is_empty() { ".".to_owned() } else { cwd };
    let mut command = Command::new(&check.run[0]);
    command
        .args(&check.run[1..])
        .current_dir(&cwd_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = match command.group_spawn() {
        Ok(child) => child,
        Err(error) => {
            return CheckReport {
                id: check.id.clone(),
                argv: check.run.clone(),
                cwd,
                verdict: Verdict::Inconclusive,
                exit_code: None,
                duration_ms: (Utc::now() - started).num_milliseconds(),
                stdout_bytes: 0,
                stderr_bytes: 0,
                stdout_sha256: crate::integrity::digest(&[]),
                stderr_sha256: crate::integrity::digest(&[]),
                stdout: check.retain_output.then(String::new),
                stderr: check.retain_output.then(String::new),
                output_retained: check.retain_output,
                output_truncated: false,
                note: Some(format!("could not start command: {error}")),
            };
        }
    };
    let stdout = child.inner().stdout.take().expect("piped stdout");
    let stderr = child.inner().stderr.take().expect("piped stderr");
    let retain_output = check.retain_output;
    let stdout_reader = thread::spawn(move || read_stream(stdout, retain_output));
    let stderr_reader = thread::spawn(move || read_stream(stderr, retain_output));
    let waited = wait_group_timeout(
        &mut child,
        Duration::from_secs(check.timeout_seconds),
        || stdout_reader.is_finished() && stderr_reader.is_finished(),
    );
    let (mut verdict, exit_code, mut note) = match waited {
        Ok(Some(status)) if status.success() => (Verdict::Pass, status.code(), None),
        Ok(Some(status)) => (
            Verdict::Fail,
            status.code(),
            Some(format!("command exited with {status}")),
        ),
        Ok(None) => {
            let kill_note = child
                .kill()
                .and_then(|_| child.wait())
                .map(|_| String::new())
                .unwrap_or_else(|error| format!("; process cleanup failed: {error}"));
            (
                Verdict::Inconclusive,
                None,
                Some(format!(
                    "timed out after {} seconds{kill_note}",
                    check.timeout_seconds
                )),
            )
        }
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            (
                Verdict::Inconclusive,
                None,
                Some(format!("could not wait for command: {error}")),
            )
        }
    };
    if !stdout_reader.is_finished() || !stderr_reader.is_finished() {
        wait_until(Duration::from_secs(1), || {
            stdout_reader.is_finished() && stderr_reader.is_finished()
        });
    }
    let stdout = join_capture(stdout_reader, check.retain_output, "stdout");
    let stderr = join_capture(stderr_reader, check.retain_output, "stderr");
    let mut capture_errors = Vec::new();
    let stdout = stdout.unwrap_or_else(|error| {
        capture_errors.push(error);
        StreamCapture::empty(check.retain_output)
    });
    let stderr = stderr.unwrap_or_else(|error| {
        capture_errors.push(error);
        StreamCapture::empty(check.retain_output)
    });
    if !capture_errors.is_empty() {
        verdict = Verdict::Inconclusive;
        let detail = format!("evidence capture incomplete: {}", capture_errors.join("; "));
        note = Some(match note {
            Some(existing) => format!("{existing}; {detail}"),
            None => detail,
        });
    }
    CheckReport {
        id: check.id.clone(),
        argv: check.run.clone(),
        cwd,
        verdict,
        exit_code,
        duration_ms: (Utc::now() - started).num_milliseconds(),
        stdout_bytes: stdout.bytes,
        stderr_bytes: stderr.bytes,
        stdout_sha256: stdout.sha256,
        stderr_sha256: stderr.sha256,
        stdout: stdout.retained,
        stderr: stderr.retained,
        output_retained: check.retain_output,
        output_truncated: stdout.truncated || stderr.truncated,
        note,
    }
}

fn join_capture(
    reader: thread::JoinHandle<io::Result<StreamCapture>>,
    _retain_output: bool,
    stream: &str,
) -> std::result::Result<StreamCapture, String> {
    if reader.is_finished() {
        reader
            .join()
            .map_err(|_| format!("{stream} reader panicked"))?
            .map_err(|error| format!("cannot read {stream}: {error}"))
    } else {
        // Cleanup has exceeded its grace period. The inconclusive verdict is
        // authoritative; detach instead of hanging the Base process.
        Err(format!("{stream} reader exceeded cleanup deadline"))
    }
}

#[derive(Debug, Default)]
struct StreamCapture {
    bytes: u64,
    sha256: String,
    retained: Option<String>,
    truncated: bool,
}

impl StreamCapture {
    fn empty(retain: bool) -> Self {
        Self {
            bytes: 0,
            sha256: crate::integrity::digest(&[]),
            retained: retain.then(String::new),
            truncated: false,
        }
    }
}

fn read_stream(mut reader: impl Read, retain: bool) -> io::Result<StreamCapture> {
    let mut retained = Vec::new();
    let mut buffer = [0_u8; 8192];
    let mut truncated = false;
    let mut bytes = 0_u64;
    let mut hasher = Sha256::new();
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        bytes = bytes.saturating_add(read as u64);
        hasher.update(&buffer[..read]);
        if retain {
            let remaining = MAX_CAPTURE_BYTES.saturating_sub(retained.len());
            retained.extend_from_slice(&buffer[..read.min(remaining)]);
            truncated |= read > remaining;
        }
    }
    let digest = hasher.finalize();
    let mut sha256 = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut sha256, "{byte:02x}").expect("write to string");
    }
    Ok(StreamCapture {
        bytes,
        sha256,
        retained: retain.then(|| String::from_utf8_lossy(&retained).into_owned()),
        truncated,
    })
}

struct EvidenceFile {
    file: File,
    path: std::path::PathBuf,
    relative: String,
}

fn reserve_evidence(
    project_root: &Path,
    slug: &str,
    suite: &str,
    at: chrono::DateTime<Utc>,
) -> Result<EvidenceFile> {
    validate_run_slug(slug)?;
    let run = project_root.join(".base/runs").join(slug);
    if !run.is_dir() {
        bail!("no run folder at .base/runs/{slug}");
    }
    let stem = format!(
        "{}-{}-p{}",
        suite,
        at.format("%Y%m%dT%H%M%S%.3fZ"),
        std::process::id()
    );
    let directory = run.join("evidence/verifications");
    fs::create_dir_all(&directory)
        .with_context(|| format!("cannot create {}", directory.display()))?;
    for suffix in 1..=1000 {
        let name = if suffix == 1 {
            format!("{stem}.json")
        } else {
            format!("{stem}-{suffix}.json")
        };
        let path = directory.join(name);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => {
                let relative = path
                    .strip_prefix(project_root)
                    .expect("verification evidence is below project")
                    .to_string_lossy()
                    .replace('\\', "/");
                return Ok(EvidenceFile {
                    file,
                    path,
                    relative,
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(error).with_context(|| format!("cannot reserve {}", path.display()));
            }
        }
    }
    bail!(
        "could not reserve a unique verifier evidence path under {}",
        directory.display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FailingReader {
        emitted: bool,
    }

    impl Read for FailingReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            if self.emitted {
                return Err(io::Error::other("simulated pipe failure"));
            }
            self.emitted = true;
            buffer[..7].copy_from_slice(b"partial");
            Ok(7)
        }
    }

    #[test]
    fn stream_read_failures_never_produce_partial_passing_evidence() {
        let error = read_stream(FailingReader { emitted: false }, false).unwrap_err();
        assert!(error.to_string().contains("simulated pipe failure"));
    }
}
