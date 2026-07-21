//! Inverse reader: understand another agent system well enough to design a better
//! one. This is the reverse of `render.rs` (canon → harness) and the "understand"
//! half of pack migration.
//!
//! The reader NEVER writes canon and NEVER designs the target. Migration produces a
//! capability-first, consolidated, base-native system authored by the `/migrate`
//! agent under a written architecture doctrine (docs/DECISIONS.md D-028/D-029) — it
//! is a redesign, not a 1:1 mirror. So this module's whole job is to expose the
//! source's structure clearly: the definition inventory, capability clusters and
//! redundancy signals that make over-fragmentation visible, the raw material
//! (knowledge / state / tooling / generated) classified by content type, and a
//! summary of harness config. It reports signals; the agent makes the decisions.
//!
//! Claude Code source formats verified against code.claude.com/docs on 2026-07-21.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

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

const PORTABLE_AGENT_FIELDS: &[&str] = &["name", "description", "tools", "skills", "permissionMode"];
const PORTABLE_SKILL_FIELDS: &[&str] = &["name", "description", "when_to_use", "argument-hint"];

/// Standard definition directories, never treated as bespoke content.
const DEFINITION_DIRS: &[&str] = &["agents", "skills", "commands"];

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    Plugin,
    LooseClaude,
}

/// The canon kind a source definition would become. The reader proposes; the
/// migration agent decides whether the definition survives the redesign at all.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CanonKind {
    PackManifest,
    Agent,
    Skill,
    Pipeline,
    Policy,
    Rule,
    Gate,
}

/// How faithfully a source definition maps onto a canon kind, before any redesign.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Fidelity {
    Native,
    Partial,
    Manual,
    OutOfCanon,
}

/// The content type of the bespoke material a source directory holds — what raw
/// material exists for the redesign to carry, rebuild, or drop.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    /// Durable domain facts — carry as authored knowledge.
    Knowledge,
    /// Runtime state, logs, mirrors — rebuild in base's work/runs/state, don't copy.
    State,
    /// Scripts and integrations — out of canon (D-015); reproduce only if a capability.
    Tooling,
    /// Generated reports, dashboards, scan output — out of scope; regenerate.
    Generated,
    /// Could not be classified — surfaced for human judgment.
    Unclassified,
}

#[derive(Debug, Clone, Serialize)]
pub struct Artifact {
    pub source: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<CanonKind>,
    pub fidelity: Fidelity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Tools an agent declares — used for redundancy clustering.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub claude_only_fields: Vec<String>,
}

impl Artifact {
    fn definition(source: String, name: String, target: CanonKind, fidelity: Fidelity) -> Self {
        Self {
            source,
            name,
            target: Some(target),
            fidelity,
            description: None,
            tools: Vec::new(),
            notes: Vec::new(),
            claude_only_fields: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginInfo {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mcp_servers: Vec<String>,
}

/// A group of near-duplicate definitions — the signal that makes over-fragmentation
/// visible so the redesign can consolidate. A signal, never a decision.
#[derive(Debug, Clone, Serialize)]
pub struct Cluster {
    pub kind: String,
    pub label: String,
    pub members: Vec<String>,
    pub note: String,
}

/// Permissions and MCP servers, summarized rather than enumerated. Harness config
/// is not canon (D-015); a 300-entry allowlist is one summary, not 300 artifacts.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ConfigSummary {
    pub allow: usize,
    pub deny: usize,
    pub ask: usize,
    pub allow_by_prefix: BTreeMap<String, usize>,
    /// Deny rules that read like standing-denial gates worth reproducing.
    pub gate_candidates: Vec<String>,
    pub mcp_servers: Vec<String>,
    pub sources: Vec<String>,
}

impl ConfigSummary {
    pub fn is_empty(&self) -> bool {
        self.allow == 0
            && self.deny == 0
            && self.ask == 0
            && self.mcp_servers.is_empty()
            && self.sources.is_empty()
    }
}

/// A recursive rollup of one bespoke directory's content.
#[derive(Debug, Clone, Serialize)]
pub struct DirRollup {
    pub path: String,
    pub category: Category,
    pub files: usize,
    pub bytes: u64,
    pub samples: Vec<String>,
    pub note: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Summary {
    pub agents: usize,
    pub skills: usize,
    pub commands: usize,
    pub policies: usize,
    pub other_hooks: usize,
    pub knowledge_dirs: usize,
    pub state_dirs: usize,
    pub tooling_dirs: usize,
    pub generated_dirs: usize,
    pub files_scanned: usize,
}

/// The full understanding of a source system.
#[derive(Debug, Clone, Serialize)]
pub struct Ingestion {
    pub source_kind: SourceKind,
    pub root: String,
    pub claude_dir: String,
    pub format_verified: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin: Option<PluginInfo>,
    /// Agents, skills, commands, hooks, gate candidates — the migratable core.
    pub definitions: Vec<Artifact>,
    /// Redundancy / fragmentation signals over the definitions.
    pub clusters: Vec<Cluster>,
    pub config: ConfigSummary,
    /// Bespoke directories classified by content type.
    pub content: Vec<DirRollup>,
    /// What base adds on top of a faithful reproduction.
    pub improvements: Vec<String>,
    pub summary: Summary,
}

/// Resolve the project root and the `.claude` directory from whatever the user
/// pointed at — the project root, or the `.claude` directory itself.
fn resolve_roots(path: &Path) -> (PathBuf, PathBuf) {
    // A plugin root carries its manifest; definitions sit at the root itself.
    if path.join(".claude-plugin").join("plugin.json").is_file() {
        let nested = path.join(".claude");
        let claude_dir = if nested.is_dir() { nested } else { path.to_path_buf() };
        return (path.to_path_buf(), claude_dir);
    }
    let nested = path.join(".claude");
    if nested.is_dir() {
        return (path.to_path_buf(), nested);
    }
    // Pointed at the `.claude` directory itself.
    let is_claude_dir = path.file_name().and_then(|name| name.to_str()) == Some(".claude")
        || DEFINITION_DIRS.iter().any(|kind| path.join(kind).is_dir());
    if is_claude_dir {
        let project_root = path.parent().unwrap_or(path).to_path_buf();
        return (project_root, path.to_path_buf());
    }
    (path.to_path_buf(), path.to_path_buf())
}

/// Understand the source system rooted at `path`.
pub fn ingest(path: &Path) -> Result<Ingestion> {
    if !path.is_dir() {
        anyhow::bail!("ingest source {} is not a directory", path.display());
    }
    let (project_root, claude_dir) = resolve_roots(path);

    let manifest_path = project_root.join(".claude-plugin").join("plugin.json");
    let (source_kind, plugin) = if manifest_path.is_file() {
        (SourceKind::Plugin, Some(read_plugin(&manifest_path)?))
    } else {
        (SourceKind::LooseClaude, None)
    };

    // Definitions live under either the project root (plugin layout) or the
    // `.claude` directory (loose layout); search both, de-duplicated.
    let mut bases: Vec<PathBuf> = vec![claude_dir.clone()];
    if project_root != claude_dir {
        bases.push(project_root.clone());
    }

    let mut definitions = Vec::new();
    if let Some(info) = &plugin {
        let mut manifest = Artifact::definition(
            ".claude-plugin/plugin.json".to_owned(),
            info.name.clone().unwrap_or_else(|| "plugin".to_owned()),
            CanonKind::PackManifest,
            Fidelity::Native,
        );
        manifest.description = info.description.clone();
        manifest.notes.push(
            "plugin manifest ≈ pack.md; carry author/homepage/keywords into the manifest body"
                .to_owned(),
        );
        definitions.push(manifest);
    }

    read_agents(&bases, &project_root, &mut definitions)?;
    read_skills(&bases, &project_root, &mut definitions)?;
    read_commands(&bases, &project_root, &mut definitions)?;

    let mut config = ConfigSummary::default();
    read_settings(&project_root, &claude_dir, &mut definitions, &mut config)?;
    read_mcp(&project_root, &claude_dir, &plugin, &mut config);
    read_instructions(&project_root, &claude_dir, &mut definitions);

    let clusters = cluster_definitions(&definitions);
    let (content, files_scanned) = classify_content(&claude_dir, &project_root);

    let ingestion = Ingestion {
        source_kind,
        root: display(&project_root),
        claude_dir: display(&claude_dir),
        format_verified: FORMAT_VERIFIED.to_owned(),
        plugin,
        definitions,
        clusters,
        config,
        content,
        improvements: improvements(),
        summary: Summary::default(),
    };
    Ok(finish(ingestion, files_scanned))
}

fn finish(mut ingestion: Ingestion, files_scanned: usize) -> Ingestion {
    let mut summary = Summary {
        files_scanned,
        ..Summary::default()
    };
    for def in &ingestion.definitions {
        match def.target {
            Some(CanonKind::Agent) => summary.agents += 1,
            Some(CanonKind::Skill) => summary.skills += 1,
            Some(CanonKind::Pipeline) => summary.commands += 1,
            Some(CanonKind::Policy) => summary.policies += 1,
            _ if def.name.contains("[") => summary.other_hooks += 1,
            _ => {}
        }
    }
    for dir in &ingestion.content {
        match dir.category {
            Category::Knowledge => summary.knowledge_dirs += 1,
            Category::State => summary.state_dirs += 1,
            Category::Tooling => summary.tooling_dirs += 1,
            Category::Generated => summary.generated_dirs += 1,
            Category::Unclassified => {}
        }
    }
    ingestion
        .definitions
        .sort_by(|a, b| a.source.cmp(&b.source).then(a.name.cmp(&b.name)));
    ingestion.content.sort_by(|a, b| a.path.cmp(&b.path));
    ingestion.summary = summary;
    ingestion
}

fn improvements() -> Vec<String> {
    [
        "capability-first design: consolidate over-fragmented agents into a few clear roles + pipelines",
        "work items + kanban with explicit human verdicts",
        "stage-approval gates recorded as artifacts, not utterances",
        "runs + append-only history ledger",
        "typed verifiers (pass | fail | inconclusive), never assumed success",
        "durable handoff + pickup for cross-session continuity",
        "cross-harness output — a Claude-only source also emits Codex + Copilot surfaces",
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

// --- definitions: agents / skills / commands -------------------------------

#[derive(Debug, Deserialize)]
struct ClaudeFront {
    name: Option<String>,
    description: Option<String>,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_yaml::Value>,
}

fn read_agents(bases: &[PathBuf], project_root: &Path, out: &mut Vec<Artifact>) -> Result<()> {
    for (relative, path) in markdown_files(bases, project_root, "agents") {
        let source =
            fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
        let front = parse_front(&source);
        let tools = front
            .extra
            .get("tools")
            .map(yaml_string_list)
            .unwrap_or_default();
        let claude_only = non_portable_fields(&front.extra, PORTABLE_AGENT_FIELDS);
        let mut artifact = Artifact::definition(
            relative,
            front.name.unwrap_or_else(|| file_stem(&path)),
            CanonKind::Agent,
            if claude_only.is_empty() {
                Fidelity::Native
            } else {
                Fidelity::Partial
            },
        );
        artifact.description = front.description;
        artifact.tools = tools;
        if !claude_only.is_empty() {
            artifact.notes.push(format!(
                "Claude-only agent knobs not representable in canon: {}",
                claude_only.join(", ")
            ));
        }
        artifact.claude_only_fields = claude_only;
        out.push(artifact);
    }
    Ok(())
}

fn read_skills(bases: &[PathBuf], project_root: &Path, out: &mut Vec<Artifact>) -> Result<()> {
    for base in dedup(bases) {
        let skills_root = base.join("skills");
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
            let relative = format!("{}/skills/{dir_name}/SKILL.md", rel(project_root, &base));
            let source = fs::read_to_string(&skill_md)
                .with_context(|| format!("cannot read {}", skill_md.display()))?;
            let (front, body) = match split_frontmatter(&source) {
                Ok((fm, body)) => (parse_front_str(fm), body.to_owned()),
                Err(_) => (ClaudeFront::empty(), source.clone()),
            };
            let claude_only = non_portable_fields(&front.extra, PORTABLE_SKILL_FIELDS);
            let target = if looks_like_pipeline(&body) {
                CanonKind::Pipeline
            } else {
                CanonKind::Skill
            };
            let fidelity = match (target, claude_only.is_empty()) {
                (CanonKind::Pipeline, _) => Fidelity::Manual,
                (_, true) => Fidelity::Native,
                (_, false) => Fidelity::Partial,
            };
            let mut artifact = Artifact::definition(
                relative,
                front.name.unwrap_or(dir_name),
                target,
                fidelity,
            );
            artifact.description = front.description;
            if target == CanonKind::Pipeline {
                artifact
                    .notes
                    .push("reads as a multi-step workflow — a pipeline candidate".to_owned());
            }
            if !claude_only.is_empty() {
                artifact.notes.push(format!(
                    "Claude-only skill knobs: {}",
                    claude_only.join(", ")
                ));
                artifact.claude_only_fields = claude_only;
            }
            out.push(artifact);
        }
    }
    Ok(())
}

fn read_commands(bases: &[PathBuf], project_root: &Path, out: &mut Vec<Artifact>) -> Result<()> {
    for (relative, path) in markdown_files(bases, project_root, "commands") {
        let source =
            fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
        let front = parse_front(&source);
        let body = split_frontmatter(&source)
            .map(|(_, body)| body.to_owned())
            .unwrap_or(source);
        let target = if looks_like_pipeline(&body) {
            CanonKind::Pipeline
        } else {
            CanonKind::Skill
        };
        let mut artifact = Artifact::definition(
            relative,
            front.name.unwrap_or_else(|| file_stem(&path)),
            target,
            Fidelity::Partial,
        );
        artifact.description = front.description;
        artifact
            .notes
            .push("legacy .claude/commands entry (merged into skills v2.1.145+)".to_owned());
        out.push(artifact);
    }
    Ok(())
}

/// A body that reads as an ordered, gated procedure is a pipeline candidate.
fn looks_like_pipeline(body: &str) -> bool {
    let ordered_steps = body
        .lines()
        .filter(|line| {
            let head = line.trim_start().split('.').next().unwrap_or("");
            !head.is_empty() && head.len() <= 2 && head.chars().all(|c| c.is_ascii_digit())
        })
        .count();
    let lower = body.to_ascii_lowercase();
    let workflow_words = ["## stage", "approval gate", "independent review", "hand off", "pipeline"]
        .iter()
        .any(|needle| lower.contains(needle));
    ordered_steps >= 3 || workflow_words
}

// --- settings.json: hooks + permissions ------------------------------------

fn read_settings(
    project_root: &Path,
    claude_dir: &Path,
    definitions: &mut Vec<Artifact>,
    config: &mut ConfigSummary,
) -> Result<()> {
    let candidates = [
        claude_dir.join("settings.json"),
        claude_dir.join("settings.local.json"),
        project_root.join(".claude/settings.json"),
        project_root.join(".claude/settings.local.json"),
        claude_dir.join("hooks/hooks.json"),
    ];
    let mut seen = BTreeSet::new();
    for path in candidates {
        if !path.is_file() || !seen.insert(path.clone()) {
            continue;
        }
        let relative = rel_file(project_root, &path);
        let source =
            fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
        let value: serde_json::Value = serde_json::from_str(&source)
            .with_context(|| format!("invalid JSON in {}", path.display()))?;
        parse_hooks(&relative, &value, definitions);
        summarize_permissions(&relative, &value, config);
    }
    Ok(())
}

fn parse_hooks(relative: &str, value: &serde_json::Value, out: &mut Vec<Artifact>) {
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
        let mut seq = 0;
        for group in groups.iter() {
            let matcher = group.get("matcher").and_then(|v| v.as_str()).unwrap_or("");
            for hook in group.get("hooks").and_then(|v| v.as_array()).into_iter().flatten() {
                let index = seq;
                seq += 1;
                let hook_type = hook.get("type").and_then(|v| v.as_str()).unwrap_or("command");
                let (target, fidelity, note) = match (mapped, hook_type) {
                    (Some(canon_event), "command") => (
                        Some(CanonKind::Policy),
                        Fidelity::Partial,
                        format!("event {event} → {canon_event}; author mode, argv command, failure posture"),
                    ),
                    (Some(_), other) => (
                        None,
                        Fidelity::Manual,
                        format!("hook type `{other}` has no canon equivalent"),
                    ),
                    (None, _) => (
                        None,
                        Fidelity::Manual,
                        format!("event {event} is outside the four canon lifecycle events; reproduce as tooling if needed"),
                    ),
                };
                let mut artifact = Artifact {
                    source: relative.to_owned(),
                    name: format!("{event}[{index}]"),
                    target,
                    fidelity,
                    description: None,
                    tools: Vec::new(),
                    notes: vec![note],
                    claude_only_fields: Vec::new(),
                };
                if !matcher.is_empty() {
                    artifact.notes.push(format!("matcher `{matcher}` → match-tools glob"));
                }
                out.push(artifact);
            }
        }
    }
}

fn summarize_permissions(relative: &str, value: &serde_json::Value, config: &mut ConfigSummary) {
    let Some(permissions) = value.get("permissions").and_then(|v| v.as_object()) else {
        return;
    };
    let touched = ["allow", "deny", "ask"]
        .iter()
        .any(|bucket| permissions.get(*bucket).and_then(|v| v.as_array()).is_some());
    if touched && !config.sources.iter().any(|s| s == relative) {
        config.sources.push(relative.to_owned());
    }
    if let Some(allow) = permissions.get("allow").and_then(|v| v.as_array()) {
        for rule in allow.iter().filter_map(|v| v.as_str()) {
            config.allow += 1;
            *config.allow_by_prefix.entry(prefix_of(rule)).or_default() += 1;
        }
    }
    if let Some(deny) = permissions.get("deny").and_then(|v| v.as_array()) {
        for rule in deny.iter().filter_map(|v| v.as_str()) {
            config.deny += 1;
            if is_standing_denial(rule) && !config.gate_candidates.iter().any(|r| r == rule) {
                config.gate_candidates.push(rule.to_owned());
            }
        }
    }
    if let Some(ask) = permissions.get("ask").and_then(|v| v.as_array()) {
        config.ask += ask.iter().filter_map(|v| v.as_str()).count();
    }
}

/// The tool family a permission rule grants, e.g. `Bash(...)` → `Bash`.
fn prefix_of(rule: &str) -> String {
    if let Some(name) = rule.split(['(', ':']).next() {
        if name.starts_with("mcp__") {
            return "mcp".to_owned();
        }
        if !name.is_empty() {
            return name.to_owned();
        }
    }
    "other".to_owned()
}

/// Deny rules that read like real standing-denial policy worth a base gate.
fn is_standing_denial(rule: &str) -> bool {
    let lower = rule.to_ascii_lowercase();
    ["push", "pr create", "pull_request", "force", "--force", "reset --hard", "rm -rf", "create_pull"]
        .iter()
        .any(|needle| lower.contains(needle))
}

// --- instructions (CLAUDE.md) ----------------------------------------------

fn read_instructions(project_root: &Path, claude_dir: &Path, out: &mut Vec<Artifact>) {
    let mut seen = BTreeSet::new();
    for path in [project_root.join("CLAUDE.md"), claude_dir.join("CLAUDE.md")] {
        if !path.is_file() || !seen.insert(path.clone()) {
            continue;
        }
        let bytes = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        let mut artifact = Artifact::definition(
            rel_file(project_root, &path),
            "CLAUDE.md".to_owned(),
            CanonKind::Rule,
            Fidelity::Manual,
        );
        artifact.notes.push(format!(
            "{bytes} bytes of prose; split into durable rules and reference knowledge (resolve @imports)"
        ));
        out.push(artifact);
    }
}

// --- MCP (out of canon, D-015) ---------------------------------------------

fn read_mcp(
    project_root: &Path,
    claude_dir: &Path,
    plugin: &Option<PluginInfo>,
    config: &mut ConfigSummary,
) {
    if let Some(plugin) = plugin {
        for server in &plugin.mcp_servers {
            push_unique(&mut config.mcp_servers, server.clone());
        }
    }
    for path in [project_root.join(".mcp.json"), claude_dir.join(".mcp.json")] {
        if !path.is_file() {
            continue;
        }
        if let Ok(source) = fs::read_to_string(&path)
            && let Ok(value) = serde_json::from_str::<serde_json::Value>(&source)
                && let Some(servers) = value.get("mcpServers").and_then(|v| v.as_object()) {
                    for server in servers.keys() {
                        push_unique(&mut config.mcp_servers, server.clone());
                    }
                }
    }
}

// --- capability clustering -------------------------------------------------

/// Group near-duplicate definitions so over-fragmentation is visible. Clusters by
/// shared name affix and by near-identical tool sets. Signals only — the redesign
/// decides what to consolidate.
fn cluster_definitions(definitions: &[Artifact]) -> Vec<Cluster> {
    let agents: Vec<&Artifact> = definitions
        .iter()
        .filter(|d| d.target == Some(CanonKind::Agent))
        .collect();
    let mut clusters = Vec::new();

    // Affix families (e.g. `security-*`, `*-expert`) with 3+ members.
    let mut by_prefix: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut by_suffix: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for agent in &agents {
        let parts: Vec<&str> = agent.name.split('-').collect();
        if parts.len() >= 2 {
            by_prefix.entry(parts[0].to_owned()).or_default().push(agent.name.clone());
            by_suffix
                .entry(parts[parts.len() - 1].to_owned())
                .or_default()
                .push(agent.name.clone());
        }
    }
    let mut claimed: BTreeSet<String> = BTreeSet::new();
    for (affix, members, kind) in by_prefix
        .into_iter()
        .map(|(a, m)| (a, m, "prefix"))
        .chain(by_suffix.into_iter().map(|(a, m)| (a, m, "suffix")))
    {
        if members.len() < 3 {
            continue;
        }
        // Prefer the larger family when an agent could sit in two.
        if members.iter().all(|m| claimed.contains(m)) {
            continue;
        }
        for m in &members {
            claimed.insert(m.clone());
        }
        let label = if kind == "prefix" {
            format!("{affix}-*")
        } else {
            format!("*-{affix}")
        };
        clusters.push(Cluster {
            kind: "agent".to_owned(),
            label: label.clone(),
            members: members.clone(),
            note: format!(
                "{} agents share the `{label}` name family — likely one capability; consolidate into a role + domain knowledge or a single gated pipeline",
                members.len()
            ),
        });
    }

    // Near-identical tool sets among 3+ agents (redundant scanners/experts).
    let mut tool_groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for agent in &agents {
        if agent.tools.is_empty() {
            continue;
        }
        let mut sorted = agent.tools.clone();
        sorted.sort();
        sorted.dedup();
        tool_groups.entry(sorted.join(",")).or_default().push(agent.name.clone());
    }
    for (tools, members) in tool_groups {
        if members.len() >= 3 {
            clusters.push(Cluster {
                kind: "agent".to_owned(),
                label: format!("shared-tools[{tools}]"),
                members: members.clone(),
                note: format!(
                    "{} agents declare identical tools — near-duplicates; a strong consolidation candidate",
                    members.len()
                ),
            });
        }
    }

    // Overall fragmentation signal.
    if agents.len() > 8 {
        clusters.push(Cluster {
            kind: "summary".to_owned(),
            label: "fragmentation".to_owned(),
            members: Vec::new(),
            note: format!(
                "{} agents is high for one system — assume the source is over-fragmented and design a minimal role set (analyst / implementer / reviewer + only genuinely distinct specialists)",
                agents.len()
            ),
        });
    }
    clusters
}

// --- content classification ------------------------------------------------

fn classify_content(claude_dir: &Path, project_root: &Path) -> (Vec<DirRollup>, usize) {
    let scan_root = if claude_dir.is_dir() { claude_dir } else { project_root };
    let mut rollups = Vec::new();
    let mut files_scanned = 0;
    let Ok(entries) = fs::read_dir(scan_root) else {
        return (rollups, 0);
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        if file_type.is_dir() {
            if DEFINITION_DIRS.contains(&name.as_str()) || name == ".git" || name == "node_modules" {
                continue;
            }
            let (rollup, scanned) = rollup_dir(&entry.path(), &name);
            files_scanned += scanned;
            rollups.push(rollup);
        } else if file_type.is_file() {
            if matches!(
                name.as_str(),
                "settings.json" | "settings.local.json" | "CLAUDE.md" | ".gitignore" | ".mcp.json"
            ) {
                continue;
            }
            files_scanned += 1;
            let category = classify_file(&name, dir_hint(&name).unwrap_or(Category::Unclassified));
            rollups.push(DirRollup {
                path: format!(".claude/{name}"),
                category,
                files: 1,
                bytes: entry.metadata().map(|m| m.len()).unwrap_or(0),
                samples: Vec::new(),
                note: category_note(category, 1, &[]),
            });
        }
    }
    (rollups, files_scanned)
}

fn rollup_dir(dir: &Path, name: &str) -> (DirRollup, usize) {
    let hint = dir_hint(name);
    let mut counts: BTreeMap<Category, usize> = BTreeMap::new();
    let mut files = 0usize;
    let mut bytes = 0u64;
    let mut samples = Vec::new();
    for entry in WalkDir::new(dir)
        .max_depth(6)
        .into_iter()
        .filter_entry(|e| e.file_name() != ".git" && e.file_name() != "node_modules")
        .flatten()
    {
        if !entry.file_type().is_file() {
            continue;
        }
        files += 1;
        bytes += entry.metadata().map(|m| m.len()).unwrap_or(0);
        let fname = entry.file_name().to_string_lossy();
        let category = classify_file(&fname, hint.unwrap_or(Category::Unclassified));
        *counts.entry(category).or_default() += 1;
        if samples.len() < 3
            && let Ok(rel) = entry.path().strip_prefix(dir) {
                samples.push(format!("{name}/{}", rel.to_string_lossy().replace('\\', "/")));
            }
    }
    let primary = hint
        .filter(|_| files > 0)
        .or_else(|| counts.iter().max_by_key(|(_, n)| **n).map(|(c, _)| *c))
        .unwrap_or(Category::Unclassified);
    let breakdown: Vec<(Category, usize)> = counts.into_iter().collect();
    (
        DirRollup {
            path: format!(".claude/{name}"),
            category: primary,
            files,
            bytes,
            samples,
            note: category_note(primary, files, &breakdown),
        },
        files,
    )
}

/// Top-level directory names that strongly imply a content category.
fn dir_hint(name: &str) -> Option<Category> {
    let n = name.to_ascii_lowercase();
    const KNOWLEDGE: &[&str] = &[
        "memory", "learnings", "learning", "investigations", "data", "guide", "guides", "specs",
        "spec", "plans", "planning", "research", "features", "knowledge", "reference",
        "references", "notes", "glossary", "estimates", "docs",
    ];
    const STATE: &[&str] = &[
        "ado", "audit", "handoff", "handoffs", "implementations", "timesheet", "sessions", "state",
        "runs", "history", "tasks", "workitems", "work-items", "cache", "mirror",
    ];
    const TOOLING: &[&str] = &[
        "tools", "tool", "scripts", "script", "spe", "powershell", "ps", "bin", "vendor", "lib",
        "hooks", "mcp", "utils", "tmp", "temp", "workflows",
    ];
    const GENERATED: &[&str] = &[
        "reports", "report", "security-scans", "scans", "quality", "smoke", "solution-design",
        "solution-designs", "output", "out", "dist", "generated", "artifacts", "showcase",
        "design", "v2",
    ];
    if KNOWLEDGE.contains(&n.as_str()) {
        Some(Category::Knowledge)
    } else if STATE.contains(&n.as_str()) {
        Some(Category::State)
    } else if TOOLING.contains(&n.as_str()) {
        Some(Category::Tooling)
    } else if GENERATED.contains(&n.as_str()) {
        Some(Category::Generated)
    } else {
        None
    }
}

fn classify_file(name: &str, fallback: Category) -> Category {
    let ext = Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "jsonl" => Category::State,
        "ps1" | "psm1" | "sh" | "mjs" | "cjs" | "py" | "bat" | "cmd" => Category::Tooling,
        "html" | "htm" | "css" | "png" | "jpg" | "jpeg" | "svg" | "pdf" | "xlsx" | "lock" => {
            Category::Generated
        }
        "md" | "mdx" | "txt" => {
            if fallback == Category::Unclassified {
                Category::Knowledge
            } else {
                fallback
            }
        }
        _ => fallback,
    }
}

fn category_note(category: Category, files: usize, breakdown: &[(Category, usize)]) -> String {
    let base = match category {
        Category::Knowledge => "durable knowledge — carry the still-true parts as authored knowledge",
        Category::State => "runtime state — rebuild in base work/runs/state, do not copy bytes",
        Category::Tooling => "scripts/tooling — out of canon (D-015); reproduce only if a capability",
        Category::Generated => "generated artifacts — out of scope; regenerate, do not migrate",
        Category::Unclassified => "unclassified — review before deciding",
    };
    let noun = if files == 1 { "file" } else { "files" };
    if breakdown.len() > 1 {
        let mix: Vec<String> = breakdown
            .iter()
            .map(|(c, n)| format!("{n} {}", category_word(*c)))
            .collect();
        format!("{files} {noun} ({}); {base}", mix.join(", "))
    } else {
        format!("{files} {noun}; {base}")
    }
}

fn category_word(category: Category) -> &'static str {
    match category {
        Category::Knowledge => "knowledge",
        Category::State => "state",
        Category::Tooling => "tooling",
        Category::Generated => "generated",
        Category::Unclassified => "unclassified",
    }
}

// --- helpers ---------------------------------------------------------------

impl ClaudeFront {
    fn empty() -> Self {
        Self {
            name: None,
            description: None,
            extra: BTreeMap::new(),
        }
    }
}

fn parse_front(source: &str) -> ClaudeFront {
    match split_frontmatter(source) {
        Ok((fm, _)) => parse_front_str(fm),
        Err(_) => ClaudeFront::empty(),
    }
}

fn parse_front_str(frontmatter: &str) -> ClaudeFront {
    serde_yaml::from_str(frontmatter).unwrap_or_else(|_| ClaudeFront::empty())
}

fn yaml_string_list(value: &serde_yaml::Value) -> Vec<String> {
    match value {
        serde_yaml::Value::String(s) => s
            .split([',', ' ', '\n'])
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
            .map(|t| t.to_owned())
            .collect(),
        serde_yaml::Value::Sequence(items) => items
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.trim().to_owned()))
            .filter(|s| !s.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

/// Return `(relative_path, absolute_path)` for every `*.md` directly under
/// `<base>/<kind>` for each candidate base.
fn markdown_files(bases: &[PathBuf], project_root: &Path, kind: &str) -> Vec<(String, PathBuf)> {
    let mut found = Vec::new();
    let mut seen = BTreeSet::new();
    for base in dedup(bases) {
        let dir = base.join(kind);
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|v| v.to_str()) != Some("md") {
                continue;
            }
            if !seen.insert(path.clone()) {
                continue;
            }
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            found.push((format!("{}/{kind}/{name}", rel(project_root, &base)), path.clone()));
        }
    }
    found
}

fn dedup(bases: &[PathBuf]) -> Vec<PathBuf> {
    let mut seen = BTreeSet::new();
    bases.iter().filter(|b| seen.insert((*b).clone())).cloned().collect()
}

/// A source label for a base directory relative to the project root.
fn rel(project_root: &Path, base: &Path) -> String {
    if base == project_root {
        ".".to_owned()
    } else {
        base.strip_prefix(project_root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| ".claude".to_owned())
    }
}

fn rel_file(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.file_name().unwrap_or_default().to_string_lossy().into_owned())
}

fn display(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

fn file_stem(path: &Path) -> String {
    path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

fn push_unique(list: &mut Vec<String>, value: String) {
    if !list.contains(&value) {
        list.push(value);
    }
}

fn non_portable_fields(extra: &BTreeMap<String, serde_yaml::Value>, portable: &[&str]) -> Vec<String> {
    extra
        .keys()
        .filter(|key| !portable.contains(&key.as_str()))
        .cloned()
        .collect()
}

// Ord for Category so it can key a BTreeMap during rollup.
impl PartialOrd for Category {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Category {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, content: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    /// A deliberately over-fragmented, noisy source — the case the reader must
    /// understand: a `security-*` family with identical tools, a big allowlist,
    /// and bespoke knowledge/state/tooling dirs.
    fn messy_fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        for n in ["injection", "crypto", "ssrf", "auth-bypass"] {
            write(
                &root.join(format!(".claude/agents/security-{n}.md")),
                &format!("---\nname: security-{n}\ndescription: Scan for {n}.\ntools: Read, Grep, Glob\n---\n\nScan.\n"),
            );
        }
        write(
            &root.join(".claude/agents/implementer.md"),
            "---\nname: implementer\ndescription: Build changes.\ntools: Read, Edit, Bash\n---\n\nBuild.\n",
        );
        write(
            &root.join(".claude/skills/pickup/SKILL.md"),
            "---\nname: pickup\ndescription: Resume.\n---\n\nRun context.\n",
        );
        let allow: Vec<String> = (0..50).map(|i| format!("\"Bash(cmd{i} *)\"")).collect();
        write(
            &root.join(".claude/settings.json"),
            &format!(
                "{{\n  \"permissions\": {{ \"allow\": [{}], \"deny\": [\"Bash(git push * main*)\", \"Read(secret)\"] }},\n  \"hooks\": {{ \"PreToolUse\": [ {{ \"matcher\": \"Bash\", \"hooks\": [ {{ \"type\": \"command\", \"command\": \"guard\" }} ] }} ], \"Stop\": [ {{ \"hooks\": [ {{ \"type\": \"command\", \"command\": \"x\" }} ] }} ] }}\n}}",
                allow.join(", ")
            ),
        );
        write(&root.join(".claude/memory/glossary.md"), "# Terms\n\nDA Core = ...\n");
        write(&root.join(".claude/memory/patterns.md"), "# Patterns\n\nHelix ...\n");
        write(&root.join(".claude/ado/items-cache.json"), "{}\n");
        write(&root.join(".claude/audit/2026/01/01.jsonl"), "{\"ts\":1}\n");
        write(&root.join(".claude/tools/sync.ps1"), "Write-Host hi\n");
        write(&root.join(".claude/reports/scan.html"), "<html></html>\n");
        dir
    }

    #[test]
    fn permissions_are_summarized_not_enumerated() {
        let dir = messy_fixture();
        let ing = ingest(dir.path()).unwrap();
        assert_eq!(ing.config.allow, 50);
        assert_eq!(ing.config.deny, 2);
        // The allowlist is ONE summary, not 50 definitions.
        assert!(ing.definitions.len() < 20, "definitions flooded: {}", ing.definitions.len());
        assert_eq!(ing.config.allow_by_prefix.get("Bash"), Some(&50));
        // Only the push deny is a standing-denial gate candidate.
        assert!(ing.config.gate_candidates.iter().any(|r| r.contains("push")));
        assert!(!ing.config.gate_candidates.iter().any(|r| r.contains("secret")));
    }

    #[test]
    fn over_fragmentation_surfaces_as_a_cluster() {
        let dir = messy_fixture();
        let ing = ingest(dir.path()).unwrap();
        // The four security agents share tools → a redundancy cluster.
        assert!(
            ing.clusters
                .iter()
                .any(|c| c.label.starts_with("security-") || c.label.starts_with("shared-tools")),
            "no consolidation signal in {:?}",
            ing.clusters.iter().map(|c| &c.label).collect::<Vec<_>>()
        );
    }

    #[test]
    fn bespoke_dirs_are_classified_by_content_type() {
        let dir = messy_fixture();
        let ing = ingest(dir.path()).unwrap();
        let cat = |p: &str| ing.content.iter().find(|d| d.path == p).map(|d| d.category);
        assert_eq!(cat(".claude/memory"), Some(Category::Knowledge));
        assert_eq!(cat(".claude/ado"), Some(Category::State));
        assert_eq!(cat(".claude/audit"), Some(Category::State));
        assert_eq!(cat(".claude/tools"), Some(Category::Tooling));
        assert_eq!(cat(".claude/reports"), Some(Category::Generated));
    }

    #[test]
    fn smart_root_accepts_the_claude_dir_directly() {
        let dir = messy_fixture();
        // Point at the .claude directory itself — settings.json must still be found.
        let ing = ingest(&dir.path().join(".claude")).unwrap();
        assert_eq!(ing.config.allow, 50);
        assert!(ing.definitions.iter().any(|d| d.target == Some(CanonKind::Agent)));
    }

    #[test]
    fn plugin_manifest_is_detected() {
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
        let ing = ingest(root).unwrap();
        assert_eq!(ing.source_kind, SourceKind::Plugin);
        assert_eq!(ing.plugin.as_ref().unwrap().name.as_deref(), Some("mck"));
        assert!(ing.definitions.iter().any(|d| d.target == Some(CanonKind::PackManifest)));
        assert!(ing.definitions.iter().any(|d| d.name == "analyst"));
    }

    #[test]
    fn pipeline_heuristic_flags_multi_step_bodies() {
        assert!(looks_like_pipeline("1. do this\n2. then this\n3. finally this\n"));
        assert!(looks_like_pipeline("## Stage one\nwork\n"));
        assert!(!looks_like_pipeline("Just run one command.\n"));
    }
}
