use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::Utc;

use crate::cli::IngestArgs;
use crate::ingest::{self, CanonKind, Category, Ingestion};

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
        if let Some((json_path, md_path)) = &evidence {
            println!("evidence: {json_path} and {md_path}");
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
    let s = &ingestion.summary;
    let _ = writeln!(out, "# System understanding: {}", ingestion.root);
    let _ = writeln!(out);
    let _ = writeln!(out, "- source: {} ({kind})", ingestion.root);
    let _ = writeln!(out, "- claude dir: {}", ingestion.claude_dir);
    let _ = writeln!(out, "- Claude Code formats verified: {}", ingestion.format_verified);
    if let Some(plugin) = &ingestion.plugin {
        let _ = writeln!(
            out,
            "- plugin: {} {}",
            plugin.name.as_deref().unwrap_or("(unnamed)"),
            plugin.version.as_deref().unwrap_or("")
        );
    }
    let _ = writeln!(
        out,
        "- inventory: {} agents, {} skills, {} commands, {} policies (+{} other hooks); {} files scanned",
        s.agents, s.skills, s.commands, s.policies, s.other_hooks, s.files_scanned
    );
    let _ = writeln!(
        out,
        "- raw material: {} knowledge, {} state, {} tooling, {} generated dir(s)",
        s.knowledge_dirs, s.state_dirs, s.tooling_dirs, s.generated_dirs
    );

    // Capability signals — the point that drives consolidation.
    if !ingestion.clusters.is_empty() {
        let _ = writeln!(out, "\n## Capability signals (source is likely over-fragmented)\n");
        for cluster in &ingestion.clusters {
            if cluster.members.is_empty() {
                let _ = writeln!(out, "- {}", cluster.note);
            } else {
                let _ = writeln!(out, "- `{}`: {} — {}", cluster.label, cluster.members.join(", "), cluster.note);
            }
        }
    }

    // Definitions — the migratable core, grouped by kind.
    let _ = writeln!(out, "\n## Definitions (migratable core — redesign, don't mirror)\n");
    write_defs(&mut out, ingestion, "agents", CanonKind::Agent);
    write_defs(&mut out, ingestion, "skills", CanonKind::Skill);
    write_defs(&mut out, ingestion, "commands/pipelines", CanonKind::Pipeline);
    write_defs(&mut out, ingestion, "policies (hooks)", CanonKind::Policy);
    write_defs(&mut out, ingestion, "rules (CLAUDE.md)", CanonKind::Rule);
    let other_hooks: Vec<&_> = ingestion
        .definitions
        .iter()
        .filter(|d| d.target.is_none() && d.name.contains('['))
        .collect();
    if !other_hooks.is_empty() {
        let _ = writeln!(out, "### hooks with no canon lifecycle event ({})", other_hooks.len());
        for hook in other_hooks {
            let _ = writeln!(out, "- `{}` ({})", hook.name, first_note(hook));
        }
        let _ = writeln!(out);
    }

    // Harness config — summarized, never canon.
    let c = &ingestion.config;
    if !c.is_empty() {
        let _ = writeln!(out, "## Harness config (not canon — summarized, D-015)\n");
        if c.allow + c.deny + c.ask > 0 {
            let breakdown: Vec<String> = c
                .allow_by_prefix
                .iter()
                .map(|(prefix, n)| format!("{prefix} {n}"))
                .collect();
            let _ = writeln!(
                out,
                "- permissions: {} allow ({}), {} deny, {} ask",
                c.allow,
                if breakdown.is_empty() { "—".to_owned() } else { breakdown.join(", ") },
                c.deny,
                c.ask
            );
            if !c.gate_candidates.is_empty() {
                let _ = writeln!(out, "  - standing-denial gate candidates: {}", c.gate_candidates.join("; "));
            }
        }
        if !c.mcp_servers.is_empty() {
            let _ = writeln!(out, "- MCP servers (harness config, not canon): {}", c.mcp_servers.join(", "));
        }
        let _ = writeln!(out);
    }

    // Raw material — classified for carry / rebuild / drop.
    if !ingestion.content.is_empty() {
        let _ = writeln!(out, "## Raw material (classify: carry / rebuild / drop)\n");
        for (title, category) in [
            ("knowledge — carry the still-true parts", Category::Knowledge),
            ("state/runtime — rebuild, don't copy", Category::State),
            ("tooling — out of canon", Category::Tooling),
            ("generated — out of scope", Category::Generated),
            ("unclassified — review", Category::Unclassified),
        ] {
            let dirs: Vec<&_> = ingestion.content.iter().filter(|d| d.category == category).collect();
            if dirs.is_empty() {
                continue;
            }
            let _ = writeln!(out, "### {title} ({})", dirs.len());
            for dir in dirs {
                let _ = writeln!(out, "- `{}` — {}", dir.path, dir.note);
            }
            let _ = writeln!(out);
        }
    }

    let _ = writeln!(out, "## What base adds\n");
    for item in &ingestion.improvements {
        let _ = writeln!(out, "- {item}");
    }
    out
}

fn write_defs(out: &mut String, ingestion: &Ingestion, title: &str, kind: CanonKind) {
    let defs: Vec<&_> = ingestion
        .definitions
        .iter()
        .filter(|d| d.target == Some(kind))
        .collect();
    if defs.is_empty() {
        return;
    }
    let _ = writeln!(out, "### {title} ({})", defs.len());
    for def in defs {
        let fidelity = format!("{:?}", def.fidelity).to_lowercase();
        match &def.description {
            Some(desc) => {
                let _ = writeln!(out, "- `{}` [{fidelity}] — {}", def.name, truncate(desc, 100));
            }
            None => {
                let _ = writeln!(out, "- `{}` [{fidelity}]", def.name);
            }
        }
    }
    let _ = writeln!(out);
}

fn first_note(def: &ingest::Artifact) -> String {
    def.notes.first().cloned().unwrap_or_default()
}

fn truncate(text: &str, max: usize) -> String {
    let text = text.trim().replace('\n', " ");
    if text.chars().count() <= max {
        text
    } else {
        let cut: String = text.chars().take(max).collect();
        format!("{cut}…")
    }
}
