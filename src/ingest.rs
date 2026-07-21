//! Inverse reader: model another agent system's harness surfaces into a portable
//! inventory plus a canon mapping/fidelity report. This is the reverse of
//! `render.rs` (canon → harness) and the "understand" half of pack migration.
//!
//! The reader NEVER writes canon. Promotion into canon is an authored rewrite,
//! not a byte copy (docs/DECISIONS.md D-019/D-023), so this module only reports
//! what a source system contains and how faithfully each part maps to a canon
//! kind. Claude Code surfaces have outgrown vendor-neutral canon (subagents carry
//! ~16 frontmatter fields; hooks span ~30 events and 5 hook types), so honest
//! fidelity reporting — never silently dropping anything — is the point.
//!
//! Claude Code source formats verified against code.claude.com/docs on 2026-07-21.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::canon::split_frontmatter;

/// The date the Claude Code source formats this reader targets were verified.
/// Mirror `docs/ADAPTERS.md` discipline: re-verify before changing the mapping.
pub const FORMAT_VERIFIED: &str = "2026-07-21";

/// The four lifecycle events a canon policy can bind (`PolicyEvent`). Every other
/// Claude hook event is reported but not migratable to a canon policy.
const MAPPABLE_HOOK_EVENTS: &[(&str, &str)] = &[
    ("SessionStart", "session-start"),
    ("PreToolUse", "pre-tool-use"),
    ("PostToolUse", "post-tool-use"),
    ("SessionEnd", "session-end"),
];

/// Portable subagent frontmatter fields base can represent. Everything else is a
/// Claude-only knob surfaced under `claude_only_fields`.
const PORTABLE_AGENT_FIELDS: &[&str] = &["name", "description", "tools", "skills", "permissionMode"];

/// Portable skill frontmatter fields. Everything else is Claude-only.
const PORTABLE_SKILL_FIELDS: &[&str] = &["name", "description", "when_to_use", "argument-hint"];

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    /// A `.claude-plugin/plugin.json` manifest enumerates the bundle.
    Plugin,
    /// Loose `.claude/` directories with no plugin manifest.
    LooseClaude,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CanonKind {
    PackManifest,
    Agent,
    Skill,
    Pipeline,
    Policy,
    Rule,
    Knowledge,
    Gate,
}

/// How faithfully a source artifact maps onto a canon kind.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Fidelity {
    /// Maps cleanly with no vendor-specific loss.
    Native,
    /// Maps, but some source detail cannot be represented and is reported.
    Partial,
    /// Requires an authored decision (e.g. rule split, skill-vs-pipeline).
    Manual,
    /// Deliberately outside canon (e.g. MCP registration, D-015).
    OutOfCanon,
}

#[derive(Debug, Clone, Serialize)]
pub struct Artifact {
    /// Source path relative to the ingest root, or a logical identifier.
    pub source: String,
    pub name: String,
    /// The canon kind this maps to, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<CanonKind>,
    pub fidelity: Fidelity,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    /// Non-portable frontmatter fields present on the source artifact.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub claude_only_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginInfo {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    /// Server names declared inline in the plugin manifest (out of canon, D-015).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mcp_servers: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Summary {
    pub artifacts: usize,
    pub native: usize,
    pub partial: usize,
    pub manual: usize,
    pub out_of_canon: usize,
    pub claude_only_surfaces: usize,
    pub unmapped: usize,
}

/// The full result of understanding a source system.
#[derive(Debug, Clone, Serialize)]
pub struct Ingestion {
    pub source_kind: SourceKind,
    pub root: String,
    pub format_verified: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin: Option<PluginInfo>,
    pub artifacts: Vec<Artifact>,
    /// Files under recognized containers that the reader did not recognize —
    /// surfaced so migration never silently drops anything.
    pub unmapped: Vec<String>,
    /// What base adds on top of a faithful reproduction.
    pub improvements: Vec<String>,
    pub summary: Summary,
}

impl Ingestion {
    fn finish(mut self) -> Self {
        let mut summary = Summary {
            unmapped: self.unmapped.len(),
            ..Summary::default()
        };
        for artifact in &self.artifacts {
            summary.artifacts += 1;
            match artifact.fidelity {
                Fidelity::Native => summary.native += 1,
                Fidelity::Partial => summary.partial += 1,
                Fidelity::Manual => summary.manual += 1,
                Fidelity::OutOfCanon => summary.out_of_canon += 1,
            }
            if !artifact.claude_only_fields.is_empty() {
                summary.claude_only_surfaces += 1;
            }
        }
        self.artifacts.sort_by(|a, b| a.source.cmp(&b.source));
        self.unmapped.sort();
        self.summary = summary;
        self
    }
}

/// Understand the source system rooted at `root`.
pub fn ingest(root: &Path) -> Result<Ingestion> {
    if !root.is_dir() {
        anyhow::bail!("ingest source {} is not a directory", root.display());
    }
    let manifest_path = root.join(".claude-plugin").join("plugin.json");
    let (source_kind, plugin) = if manifest_path.is_file() {
        (SourceKind::Plugin, Some(read_plugin(&manifest_path)?))
    } else {
        (SourceKind::LooseClaude, None)
    };

    let mut artifacts = Vec::new();
    let mut unmapped = Vec::new();

    if let Some(info) = &plugin {
        artifacts.push(Artifact {
            source: ".claude-plugin/plugin.json".to_owned(),
            name: info.name.clone().unwrap_or_else(|| "plugin".to_owned()),
            target: Some(CanonKind::PackManifest),
            fidelity: Fidelity::Native,
            notes: vec![
                "plugin manifest maps near 1:1 to pack.md; carry author/homepage/keywords into the manifest body as provenance".to_owned(),
            ],
            claude_only_fields: Vec::new(),
        });
    }

    read_agents(root, &mut artifacts)?;
    read_skills(root, &mut artifacts)?;
    read_commands(root, &mut artifacts)?;
    read_settings(root, &mut artifacts)?;
    read_instructions(root, &mut artifacts);
    read_mcp(root, &plugin, &mut artifacts)?;
    collect_unmapped(root, &mut unmapped);

    let ingestion = Ingestion {
        source_kind,
        root: root.display().to_string().replace('\\', "/"),
        format_verified: FORMAT_VERIFIED.to_owned(),
        plugin,
        artifacts,
        unmapped,
        improvements: improvements(),
        summary: Summary::default(),
    };
    Ok(ingestion.finish())
}

fn improvements() -> Vec<String> {
    [
        "work items + kanban with explicit human verdicts",
        "stage-approval gates recorded as artifacts, not utterances",
        "runs + append-only history ledger",
        "typed verifiers (pass | fail | inconclusive), never assumed success",
        "durable handoff + pickup for cross-session continuity",
        "cross-harness compilation — a Claude-only source also emits Codex + Copilot surfaces",
        "drift-protected generated output via the sync manifest",
    ]
    .iter()
    .map(|item| (*item).to_owned())
    .collect()
}

// --- plugin manifest -------------------------------------------------------

#[derive(Debug, Deserialize)]
struct PluginManifest {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    #[serde(rename = "mcpServers", default)]
    mcp_servers: Option<serde_json::Value>,
}

fn read_plugin(path: &Path) -> Result<PluginInfo> {
    let source =
        fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;
    let manifest: PluginManifest = serde_json::from_str(&source)
        .with_context(|| format!("invalid plugin manifest JSON in {}", path.display()))?;
    let mcp_servers = manifest
        .mcp_servers
        .as_ref()
        .and_then(|value| value.as_object())
        .map(|object| object.keys().cloned().collect())
        .unwrap_or_default();
    Ok(PluginInfo {
        name: manifest.name,
        version: manifest.version,
        description: manifest.description,
        mcp_servers,
    })
}

// --- agents ----------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct ClaudeAgentFront {
    name: Option<String>,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_yaml::Value>,
}

fn read_agents(root: &Path, artifacts: &mut Vec<Artifact>) -> Result<()> {
    for (relative, path) in markdown_files(root, "agents") {
        let source =
            fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
        let Ok((frontmatter, _body)) = split_frontmatter(&source) else {
            artifacts.push(Artifact {
                source: relative,
                name: file_stem(&path),
                target: Some(CanonKind::Agent),
                fidelity: Fidelity::Manual,
                notes: vec!["no YAML frontmatter; author the agent by hand".to_owned()],
                claude_only_fields: Vec::new(),
            });
            continue;
        };
        let front: ClaudeAgentFront = serde_yaml::from_str(frontmatter).unwrap_or(ClaudeAgentFront {
            name: None,
            extra: BTreeMap::new(),
        });
        let claude_only = non_portable_fields(&front.extra, PORTABLE_AGENT_FIELDS);
        let name = front.name.unwrap_or_else(|| file_stem(&path));
        let mut notes = Vec::new();
        if front.extra.contains_key("permissionMode") {
            notes.push(
                "map permissionMode: plan → access: read-only; other modes need review".to_owned(),
            );
        }
        let fidelity = if claude_only.is_empty() {
            Fidelity::Native
        } else {
            notes.push(format!(
                "Claude-only agent knobs not representable in canon: {}",
                claude_only.join(", ")
            ));
            Fidelity::Partial
        };
        artifacts.push(Artifact {
            source: relative,
            name,
            target: Some(CanonKind::Agent),
            fidelity,
            notes,
            claude_only_fields: claude_only,
        });
    }
    Ok(())
}

// --- skills ----------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct ClaudeSkillFront {
    name: Option<String>,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_yaml::Value>,
}

fn read_skills(root: &Path, artifacts: &mut Vec<Artifact>) -> Result<()> {
    for base in ["skills", ".claude/skills"] {
        let skills_root = root.join(base.replace('/', std::path::MAIN_SEPARATOR_STR));
        if !skills_root.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&skills_root)
            .with_context(|| format!("cannot read {}", skills_root.display()))?
        {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let skill_md = entry.path().join("SKILL.md");
            if !skill_md.is_file() {
                continue;
            }
            let dir_name = entry.file_name().to_string_lossy().into_owned();
            let relative = format!("{base}/{dir_name}/SKILL.md");
            let source = fs::read_to_string(&skill_md)
                .with_context(|| format!("cannot read {}", skill_md.display()))?;
            let (name, body, claude_only) = match split_frontmatter(&source) {
                Ok((frontmatter, body)) => {
                    let front: ClaudeSkillFront =
                        serde_yaml::from_str(frontmatter).unwrap_or(ClaudeSkillFront {
                            name: None,
                            extra: BTreeMap::new(),
                        });
                    let claude_only = non_portable_fields(&front.extra, PORTABLE_SKILL_FIELDS);
                    (front.name.unwrap_or_else(|| dir_name.clone()), body.to_owned(), claude_only)
                }
                Err(_) => (dir_name.clone(), source.clone(), Vec::new()),
            };
            let mut notes = Vec::new();
            let target = if looks_like_pipeline(&body) {
                notes.push(
                    "reads as a multi-step workflow — author as a pipeline with stages, gates, and a verifier".to_owned(),
                );
                CanonKind::Pipeline
            } else {
                CanonKind::Skill
            };
            let fidelity = match target {
                CanonKind::Pipeline => Fidelity::Manual,
                _ if claude_only.is_empty() => Fidelity::Native,
                _ => {
                    notes.push(format!(
                        "Claude-only skill knobs not representable in canon: {}",
                        claude_only.join(", ")
                    ));
                    Fidelity::Partial
                }
            };
            artifacts.push(Artifact {
                source: relative,
                name,
                target: Some(target),
                fidelity,
                notes,
                claude_only_fields: claude_only,
            });
        }
    }
    Ok(())
}

/// A skill body that reads as an ordered, gated procedure is better authored as a
/// pipeline than a single skill. Conservative heuristic; flags for human review.
fn looks_like_pipeline(body: &str) -> bool {
    let ordered_steps = body
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            matches!(trimmed.split('.').next(), Some(head) if head.len() <= 2 && head.chars().all(|c| c.is_ascii_digit()) && !head.is_empty())
        })
        .count();
    let workflow_words = ["## stage", "approval gate", "independent review", "hand off"]
        .iter()
        .any(|needle| body.to_ascii_lowercase().contains(needle));
    ordered_steps >= 3 || workflow_words
}

// --- legacy slash commands -------------------------------------------------

fn read_commands(root: &Path, artifacts: &mut Vec<Artifact>) -> Result<()> {
    for (relative, path) in markdown_files(root, "commands") {
        let source =
            fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
        let body = split_frontmatter(&source)
            .map(|(_, body)| body.to_owned())
            .unwrap_or(source);
        let target = if looks_like_pipeline(&body) {
            CanonKind::Pipeline
        } else {
            CanonKind::Skill
        };
        artifacts.push(Artifact {
            source: relative,
            name: file_stem(&path),
            target: Some(target),
            fidelity: Fidelity::Manual,
            notes: vec![
                "legacy .claude/commands entry (merged into skills v2.1.145+); a same-named skill wins".to_owned(),
            ],
            claude_only_fields: Vec::new(),
        });
    }
    Ok(())
}

// --- settings.json: hooks + permissions ------------------------------------

fn read_settings(root: &Path, artifacts: &mut Vec<Artifact>) -> Result<()> {
    let candidates = [
        ".claude/settings.json",
        ".claude/settings.local.json",
        "hooks/hooks.json",
    ];
    for relative in candidates {
        let path = root.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
        if !path.is_file() {
            continue;
        }
        let source =
            fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
        let value: serde_json::Value = serde_json::from_str(&source)
            .with_context(|| format!("invalid JSON in {}", path.display()))?;
        parse_hooks(relative, &value, artifacts);
        parse_permissions(relative, &value, artifacts);
    }
    Ok(())
}

fn parse_hooks(relative: &str, value: &serde_json::Value, artifacts: &mut Vec<Artifact>) {
    let Some(hooks) = value.get("hooks").and_then(|value| value.as_object()) else {
        return;
    };
    for (event, groups) in hooks {
        if event == "disableAllHooks" {
            continue;
        }
        let mapped = MAPPABLE_HOOK_EVENTS
            .iter()
            .find(|(name, _)| name == event)
            .map(|(_, canon)| *canon);
        let Some(groups) = groups.as_array() else {
            continue;
        };
        for (index, group) in groups.iter().enumerate() {
            let matcher = group
                .get("matcher")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .to_owned();
            let entries = group.get("hooks").and_then(|value| value.as_array());
            for hook in entries.into_iter().flatten() {
                let hook_type = hook
                    .get("type")
                    .and_then(|value| value.as_str())
                    .unwrap_or("command");
                let name = format!("{event}[{index}]");
                let (target, fidelity, mut notes) = match (mapped, hook_type) {
                    (Some(canon_event), "command") => (
                        Some(CanonKind::Policy),
                        Fidelity::Partial,
                        vec![format!(
                            "event {event} → {canon_event}; author mode (context/guard/observe), argv command, and failure posture"
                        )],
                    ),
                    (Some(_), other) => (
                        None,
                        Fidelity::Manual,
                        vec![format!(
                            "hook type `{other}` has no canon equivalent (canon policies invoke an argv command)"
                        )],
                    ),
                    (None, _) => (
                        None,
                        Fidelity::Manual,
                        vec![format!(
                            "event {event} is outside the four canon-mappable lifecycle events; reported, not migrated"
                        )],
                    ),
                };
                if !matcher.is_empty() {
                    notes.push(format!("matcher `{matcher}` → match-tools glob"));
                }
                artifacts.push(Artifact {
                    source: relative.to_owned(),
                    name,
                    target,
                    fidelity,
                    notes,
                    claude_only_fields: Vec::new(),
                });
            }
        }
    }
}

fn parse_permissions(relative: &str, value: &serde_json::Value, artifacts: &mut Vec<Artifact>) {
    let Some(permissions) = value.get("permissions").and_then(|value| value.as_object()) else {
        return;
    };
    for (bucket, target, note) in [
        (
            "deny",
            Some(CanonKind::Gate),
            "deny rule → candidate standing-denial gate (e.g. never-push-default-branch)",
        ),
        (
            "allow",
            None,
            "allow rule is harness permission config, not canon; reported for context",
        ),
        (
            "ask",
            None,
            "ask rule is harness permission config, not canon; reported for context",
        ),
    ] {
        let Some(rules) = permissions.get(bucket).and_then(|value| value.as_array()) else {
            continue;
        };
        for rule in rules {
            let Some(rule) = rule.as_str() else { continue };
            artifacts.push(Artifact {
                source: relative.to_owned(),
                name: format!("permissions.{bucket}: {rule}"),
                target,
                fidelity: Fidelity::Manual,
                notes: vec![note.to_owned()],
                claude_only_fields: Vec::new(),
            });
        }
    }
}

// --- instructions (CLAUDE.md) ----------------------------------------------

fn read_instructions(root: &Path, artifacts: &mut Vec<Artifact>) {
    for relative in ["CLAUDE.md", ".claude/CLAUDE.md"] {
        let path = root.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR));
        if !path.is_file() {
            continue;
        }
        let bytes = fs::metadata(&path).map(|meta| meta.len()).unwrap_or(0);
        artifacts.push(Artifact {
            source: relative.to_owned(),
            name: relative.to_owned(),
            target: Some(CanonKind::Rule),
            fidelity: Fidelity::Manual,
            notes: vec![format!(
                "{bytes} bytes of prose; split into durable rules and reference knowledge as an authored rewrite (also resolve @imports)"
            )],
            claude_only_fields: Vec::new(),
        });
    }
}

// --- MCP (out of canon, D-015) ---------------------------------------------

fn read_mcp(root: &Path, plugin: &Option<PluginInfo>, artifacts: &mut Vec<Artifact>) -> Result<()> {
    if let Some(plugin) = plugin {
        for server in &plugin.mcp_servers {
            artifacts.push(Artifact {
                source: ".claude-plugin/plugin.json".to_owned(),
                name: server.clone(),
                target: None,
                fidelity: Fidelity::OutOfCanon,
                notes: vec![
                    "plugin-inline MCP registration stays harness config, not canon (D-015)"
                        .to_owned(),
                ],
                claude_only_fields: Vec::new(),
            });
        }
    }
    let path = root.join(".mcp.json");
    if path.is_file() {
        let source = fs::read_to_string(&path)
            .with_context(|| format!("cannot read {}", path.display()))?;
        let servers = serde_json::from_str::<serde_json::Value>(&source)
            .ok()
            .and_then(|value| {
                value
                    .get("mcpServers")
                    .and_then(|value| value.as_object())
                    .map(|object| object.keys().cloned().collect::<Vec<_>>())
            })
            .unwrap_or_default();
        for server in servers {
            artifacts.push(Artifact {
                source: ".mcp.json".to_owned(),
                name: server,
                target: None,
                fidelity: Fidelity::OutOfCanon,
                notes: vec![
                    "MCP registration stays harness config, admitted by rule, not canon (D-015)".to_owned(),
                ],
                claude_only_fields: Vec::new(),
            });
        }
    }
    Ok(())
}

// --- unmapped sweep --------------------------------------------------------

fn collect_unmapped(root: &Path, unmapped: &mut Vec<String>) {
    let recognized = [
        "agents",
        "skills",
        "commands",
        "settings.json",
        "settings.local.json",
        "CLAUDE.md",
    ];
    let claude_dir = root.join(".claude");
    if let Ok(entries) = fs::read_dir(&claude_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !recognized.contains(&name.as_str()) {
                unmapped.push(format!(".claude/{name}"));
            }
        }
    }
}

// --- helpers ---------------------------------------------------------------

/// Return `(relative_path, absolute_path)` for every `*.md` directly under
/// `<root>/<kind>` and `<root>/.claude/<kind>`.
fn markdown_files(root: &Path, kind: &str) -> Vec<(String, std::path::PathBuf)> {
    let mut found = Vec::new();
    for base in [kind.to_owned(), format!(".claude/{kind}")] {
        let dir = root.join(base.replace('/', std::path::MAIN_SEPARATOR_STR));
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("md") {
                continue;
            }
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            found.push((format!("{base}/{name}"), path.clone()));
        }
    }
    found
}

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn non_portable_fields(
    extra: &BTreeMap<String, serde_yaml::Value>,
    portable: &[&str],
) -> Vec<String> {
    extra
        .keys()
        .filter(|key| !portable.contains(&key.as_str()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, content: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // An agent that mirrors base's own rendered Claude surface plus a
        // Claude-only knob, so the partial bucket must catch it.
        write(
            &root.join(".claude/agents/reviewer.md"),
            "---\nname: reviewer\ndescription: Reviews changes.\ntools: Read, Grep\nmodel: opus\ncolor: blue\n---\n\nReview the diff.\n",
        );
        write(
            &root.join(".claude/skills/pickup/SKILL.md"),
            "---\nname: pickup\ndescription: Resume work.\n---\n\nRun base state context.\n",
        );
        write(
            &root.join(".claude/settings.json"),
            r#"{
  "permissions": { "deny": ["Bash(git push * main*)"] },
  "hooks": {
    "PreToolUse": [ { "matcher": "Bash", "hooks": [ { "type": "command", "command": "guard" } ] } ],
    "Notification": [ { "hooks": [ { "type": "command", "command": "notify" } ] } ]
  }
}"#,
        );
        write(&root.join("CLAUDE.md"), "# Project\n\nDo good work.\n");
        write(&root.join(".claude/unknown-thing.txt"), "mystery\n");
        dir
    }

    fn find<'a>(ingestion: &'a Ingestion, name: &str) -> &'a Artifact {
        ingestion
            .artifacts
            .iter()
            .find(|artifact| artifact.name == name)
            .unwrap_or_else(|| panic!("missing artifact {name}"))
    }

    #[test]
    fn loose_claude_project_maps_each_surface_with_honest_fidelity() {
        let dir = fixture();
        let ingestion = ingest(dir.path()).unwrap();
        assert_eq!(ingestion.source_kind, SourceKind::LooseClaude);

        let reviewer = find(&ingestion, "reviewer");
        assert_eq!(reviewer.target, Some(CanonKind::Agent));
        // model + color are Claude-only knobs → partial, never silently dropped.
        assert_eq!(reviewer.fidelity, Fidelity::Partial);
        assert!(reviewer.claude_only_fields.contains(&"model".to_owned()));
        assert!(reviewer.claude_only_fields.contains(&"color".to_owned()));

        let pickup = find(&ingestion, "pickup");
        assert_eq!(pickup.target, Some(CanonKind::Skill));
        assert_eq!(pickup.fidelity, Fidelity::Native);

        // The PreToolUse command hook maps to a policy; Notification does not.
        let pre = find(&ingestion, "PreToolUse[0]");
        assert_eq!(pre.target, Some(CanonKind::Policy));
        let notify = find(&ingestion, "Notification[0]");
        assert_eq!(notify.target, None);

        assert!(ingestion.artifacts.iter().any(|a| a.target == Some(CanonKind::Gate)));
        assert!(ingestion.artifacts.iter().any(|a| a.target == Some(CanonKind::Rule)));

        // The stray file is surfaced, not dropped.
        assert!(ingestion.unmapped.iter().any(|p| p.contains("unknown-thing")));
        assert!(ingestion.summary.partial >= 1);
        assert_eq!(ingestion.summary.unmapped, ingestion.unmapped.len());
    }

    #[test]
    fn plugin_manifest_is_detected_and_maps_to_a_pack() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(
            &root.join(".claude-plugin/plugin.json"),
            r#"{ "name": "mck", "version": "1.2.0", "description": "Client system" }"#,
        );
        write(
            &root.join("agents/analyst.md"),
            "---\nname: analyst\ndescription: Analyze.\n---\n\nAnalyze.\n",
        );
        let ingestion = ingest(root).unwrap();
        assert_eq!(ingestion.source_kind, SourceKind::Plugin);
        assert_eq!(ingestion.plugin.as_ref().unwrap().name.as_deref(), Some("mck"));
        let manifest = find(&ingestion, "mck");
        assert_eq!(manifest.target, Some(CanonKind::PackManifest));
        // The root-level agent is discovered even without a .claude/ prefix.
        assert!(ingestion.artifacts.iter().any(|a| a.name == "analyst"));
    }

    #[test]
    fn multi_step_skill_body_suggests_a_pipeline() {
        assert!(looks_like_pipeline("1. do this\n2. then this\n3. finally this\n"));
        assert!(looks_like_pipeline("## Stage one\nwork\n"));
        assert!(!looks_like_pipeline("Just run one command.\n"));
    }
}
