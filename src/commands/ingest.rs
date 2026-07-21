use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::Utc;

use crate::cli::IngestArgs;
use crate::ingest::{self, Artifact, Ingestion};

use super::{print_json, validate_run_slug};

pub fn run(project_root: &Path, args: IngestArgs, json: bool) -> Result<()> {
    let ingestion = ingest::ingest(&args.path)?;
    let report = render_report(&ingestion);

    let evidence = match args.run.as_deref() {
        Some(slug) => Some(write_evidence(project_root, slug, &ingestion, &report)?),
        None => None,
    };

    if json {
        print_json(&ingestion)?;
    } else {
        print!("{report}");
        if let Some(paths) = &evidence {
            println!("evidence: {} and {}", paths.0, paths.1);
        }
    }
    Ok(())
}

fn write_evidence(
    project_root: &Path,
    slug: &str,
    ingestion: &Ingestion,
    report: &str,
) -> Result<(String, String)> {
    validate_run_slug(slug)?;
    let run = project_root.join(".base/runs").join(slug);
    if !run.is_dir() {
        anyhow::bail!("no run folder at .base/runs/{slug}");
    }
    let directory = run.join("evidence/migration");
    fs::create_dir_all(&directory)
        .with_context(|| format!("cannot create {}", directory.display()))?;
    let stem = format!(
        "ingest-{}-p{}",
        Utc::now().format("%Y%m%dT%H%M%S%.3fZ"),
        std::process::id()
    );
    let json_path = directory.join(format!("{stem}.json"));
    let markdown_path = directory.join(format!("{stem}.md"));
    fs::write(&json_path, serde_json::to_vec_pretty(ingestion)?)
        .with_context(|| format!("cannot write {}", json_path.display()))?;
    fs::write(&markdown_path, report.as_bytes())
        .with_context(|| format!("cannot write {}", markdown_path.display()))?;
    Ok((relative(project_root, &json_path), relative(project_root, &markdown_path)))
}

fn relative(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn render_report(ingestion: &Ingestion) -> String {
    let mut out = String::new();
    let kind = match ingestion.source_kind {
        ingest::SourceKind::Plugin => "plugin",
        ingest::SourceKind::LooseClaude => "loose .claude/",
    };
    let _ = writeln!(out, "# Ingestion report");
    let _ = writeln!(out);
    let _ = writeln!(out, "- source: {} ({kind})", ingestion.root);
    let _ = writeln!(
        out,
        "- Claude Code formats verified: {}",
        ingestion.format_verified
    );
    if let Some(plugin) = &ingestion.plugin {
        let _ = writeln!(
            out,
            "- plugin: {} {}",
            plugin.name.as_deref().unwrap_or("(unnamed)"),
            plugin.version.as_deref().unwrap_or("")
        );
    }
    let summary = &ingestion.summary;
    let _ = writeln!(
        out,
        "- summary: {} artifacts ({} native, {} partial, {} manual, {} out-of-canon), {} claude-only surfaces, {} unmapped",
        summary.artifacts,
        summary.native,
        summary.partial,
        summary.manual,
        summary.out_of_canon,
        summary.claude_only_surfaces,
        summary.unmapped,
    );

    let _ = writeln!(out, "\n## Mapping\n");
    for artifact in &ingestion.artifacts {
        write_artifact(&mut out, artifact);
    }

    if !ingestion.unmapped.is_empty() {
        let _ = writeln!(out, "\n## Unmapped (review — nothing dropped silently)\n");
        for path in &ingestion.unmapped {
            let _ = writeln!(out, "- {path}");
        }
    }

    let _ = writeln!(out, "\n## Base adds\n");
    for item in &ingestion.improvements {
        let _ = writeln!(out, "- {item}");
    }
    out
}

fn write_artifact(out: &mut String, artifact: &Artifact) {
    let target = artifact
        .target
        .map(|kind| format!("{kind:?}").to_lowercase())
        .unwrap_or_else(|| "— (reported only)".to_owned());
    let fidelity = format!("{:?}", artifact.fidelity).to_lowercase();
    let _ = writeln!(
        out,
        "- `{}` → {target} [{fidelity}] ({})",
        artifact.name, artifact.source
    );
    for note in &artifact.notes {
        let _ = writeln!(out, "    - {note}");
    }
}
