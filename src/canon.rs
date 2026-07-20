use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::config::{Config, GateKind, validate_id};

#[derive(Debug, Clone)]
pub struct Canon {
    pub rules: BTreeMap<String, Rule>,
    pub agents: BTreeMap<String, Agent>,
    pub skills: BTreeMap<String, Skill>,
    pub stages: BTreeMap<String, Stage>,
    pub pipelines: BTreeMap<String, Pipeline>,
    pub policies: BTreeMap<String, Policy>,
    pub verifiers: BTreeMap<String, Verifier>,
    pub knowledge: BTreeMap<String, Knowledge>,
    pub overrides: Vec<CanonOverride>,
}

impl Canon {
    pub fn load(global_root: &Path, project_root: &Path, config: &Config) -> Result<Self> {
        let mut canon = Self {
            rules: BTreeMap::new(),
            agents: BTreeMap::new(),
            skills: BTreeMap::new(),
            stages: BTreeMap::new(),
            pipelines: BTreeMap::new(),
            policies: BTreeMap::new(),
            verifiers: BTreeMap::new(),
            knowledge: BTreeMap::new(),
            overrides: Vec::new(),
        };

        let global_canon = global_root.join("canon");
        if global_canon.is_dir() {
            canon.load_layer(&global_canon, Layer::Global, None)?;
        }
        for pack in &config.packs {
            crate::pack::verify_installed(project_root, pack)?;
            let root = crate::pack::installed_root(project_root, &pack.id);
            canon.load_layer(&root, Layer::Pack, Some(&pack.id))?;
        }
        let project_canon = project_root.join(".base/canon");
        if project_canon.is_dir() {
            canon.load_layer(&project_canon, Layer::Project, None)?;
        }

        canon.validate(config)?;
        Ok(canon)
    }

    pub fn validate(&self, config: &Config) -> Result<()> {
        if self.pipelines.is_empty() {
            bail!("canon has no pipelines");
        }
        for pipeline in self.pipelines.values() {
            if pipeline.meta.stages.is_empty() {
                bail!("pipeline `{}` has no stages", pipeline.meta.id);
            }
            for stage_ref in &pipeline.meta.stages {
                let Some(stage) = self.stages.get(&stage_ref.use_stage) else {
                    bail!(
                        "pipeline `{}` references missing stage `{}`",
                        pipeline.meta.id,
                        stage_ref.use_stage
                    );
                };
                // Stages inline into rendered skills, so a project pipeline built on a
                // global-only stage would commit bytes the repo cannot reproduce (D-018).
                if pipeline.source.repo_resident() && !stage.source.repo_resident() {
                    bail!(
                        "repo-resident pipeline `{}` references global-only stage `{}`; adopt the containing pack or copy the stage into `.base/canon/pipelines/stages/`",
                        pipeline.meta.id,
                        stage_ref.use_stage
                    );
                }
                if let Some(gate_id) = &stage_ref.gate {
                    let gate = config.gate(gate_id).with_context(|| {
                        format!(
                            "pipeline `{}` references undeclared gate `{gate_id}`",
                            pipeline.meta.id
                        )
                    })?;
                    if gate.kind != GateKind::StageApproval {
                        bail!(
                            "pipeline `{}` attaches non-stage gate `{gate_id}` to a stage",
                            pipeline.meta.id
                        );
                    }
                }
                if let Some(verifier_id) = &stage_ref.verifier {
                    let Some(verifier) = self.verifiers.get(verifier_id) else {
                        bail!(
                            "pipeline `{}` references missing verifier `{verifier_id}`",
                            pipeline.meta.id
                        );
                    };
                    if pipeline.source.repo_resident() && !verifier.source.repo_resident() {
                        bail!(
                            "repo-resident pipeline `{}` references global-only verifier `{verifier_id}`; adopt it into the repository",
                            pipeline.meta.id
                        );
                    }
                }
                if let Some(agent_id) = &stage_ref.agent {
                    let Some(agent) = self.agents.get(agent_id) else {
                        bail!(
                            "pipeline `{}` references missing agent `{agent_id}`",
                            pipeline.meta.id
                        );
                    };
                    if pipeline.source.repo_resident() && !agent.source.repo_resident() {
                        bail!(
                            "repo-resident pipeline `{}` references global-only agent `{agent_id}`; adopt it into the repository",
                            pipeline.meta.id
                        );
                    }
                }
                if stage_ref.independent_review && stage_ref.agent.is_none() {
                    bail!(
                        "pipeline `{}` marks stage `{}` as independent-review without assigning an agent",
                        pipeline.meta.id,
                        stage_ref.use_stage
                    );
                }
            }
            let final_stage = pipeline.meta.stages.last().expect("checked non-empty");
            if final_stage.use_stage != "record" {
                bail!(
                    "pipeline `{}` must end with the `record` stage so every exit is logged",
                    pipeline.meta.id
                );
            }
        }
        for agent in self.agents.values() {
            for skill_id in &agent.meta.skills {
                let Some(skill) = self.skills.get(skill_id) else {
                    bail!(
                        "agent `{}` references missing skill `{skill_id}`",
                        agent.meta.id
                    );
                };
                if agent.source.repo_resident() && !skill.source.repo_resident() {
                    bail!(
                        "repo-resident agent `{}` references global-only skill `{skill_id}`; adopt it into the repository",
                        agent.meta.id
                    );
                }
            }
        }
        for id in self.skills.keys() {
            if self.pipelines.contains_key(id)
                && self.skills[id].source.repo_resident()
                && self.pipelines[id].source.repo_resident()
            {
                bail!(
                    "skill and pipeline share id `{id}`; both compile to the same target skill path"
                );
            }
        }
        Ok(())
    }

    fn load_layer(&mut self, root: &Path, layer: Layer, pack: Option<&str>) -> Result<()> {
        let mut seen_rules = BTreeSet::new();
        load_documents::<RuleMeta>(root.join("rules"), false, |path, meta, body| {
            if !seen_rules.insert(meta.id.clone()) {
                bail!("duplicate rule id `{}` in {}", meta.id, root.display());
            }
            let id = meta.id.clone();
            let item = Rule {
                meta,
                body,
                source: Source::new(layer, pack, path),
            };
            let winner = item.source.clone();
            if let Some(previous) = self.rules.insert(id.clone(), item) {
                self.overrides
                    .push(CanonOverride::new("rule", id, previous.source, winner));
            }
            Ok(())
        })?;

        let mut seen_agents = BTreeSet::new();
        load_documents::<AgentMeta>(root.join("agents"), false, |path, meta, body| {
            if !seen_agents.insert(meta.id.clone()) {
                bail!("duplicate agent id `{}` in {}", meta.id, root.display());
            }
            validate_agent(&meta)?;
            let id = meta.id.clone();
            let item = Agent {
                meta,
                body,
                source: Source::new(layer, pack, path),
            };
            let winner = item.source.clone();
            if let Some(previous) = self.agents.insert(id.clone(), item) {
                self.overrides
                    .push(CanonOverride::new("agent", id, previous.source, winner));
            }
            Ok(())
        })?;

        self.load_skills(root, layer, pack)?;

        let mut seen_stages = BTreeSet::new();
        load_documents::<StageMeta>(root.join("pipelines/stages"), false, |path, meta, body| {
            if !seen_stages.insert(meta.id.clone()) {
                bail!("duplicate stage id `{}` in {}", meta.id, root.display());
            }
            let id = meta.id.clone();
            let item = Stage {
                meta,
                body,
                source: Source::new(layer, pack, path),
            };
            let winner = item.source.clone();
            if let Some(previous) = self.stages.insert(id.clone(), item) {
                self.overrides
                    .push(CanonOverride::new("stage", id, previous.source, winner));
            }
            Ok(())
        })?;

        let mut seen_pipelines = BTreeSet::new();
        load_documents::<PipelineMeta>(root.join("pipelines"), false, |path, meta, body| {
            if !seen_pipelines.insert(meta.id.clone()) {
                bail!("duplicate pipeline id `{}` in {}", meta.id, root.display());
            }
            let id = meta.id.clone();
            let item = Pipeline {
                meta,
                body,
                source: Source::new(layer, pack, path),
            };
            let winner = item.source.clone();
            if let Some(previous) = self.pipelines.insert(id.clone(), item) {
                self.overrides
                    .push(CanonOverride::new("pipeline", id, previous.source, winner));
            }
            Ok(())
        })?;

        let mut seen_policies = BTreeSet::new();
        load_documents::<PolicyMeta>(root.join("policies"), false, |path, meta, body| {
            if !seen_policies.insert(meta.id.clone()) {
                bail!("duplicate policy id `{}` in {}", meta.id, root.display());
            }
            validate_policy(&meta)?;
            let id = meta.id.clone();
            let item = Policy {
                meta,
                body,
                source: Source::new(layer, pack, path),
            };
            let winner = item.source.clone();
            if let Some(previous) = self.policies.insert(id.clone(), item) {
                self.overrides
                    .push(CanonOverride::new("policy", id, previous.source, winner));
            }
            Ok(())
        })?;

        let mut seen_verifiers = BTreeSet::new();
        load_documents::<VerifierMeta>(root.join("verifiers"), false, |path, meta, body| {
            if !seen_verifiers.insert(meta.id.clone()) {
                bail!("duplicate verifier id `{}` in {}", meta.id, root.display());
            }
            validate_verifier(&meta)?;
            let id = meta.id.clone();
            let item = Verifier {
                meta,
                body,
                source: Source::new(layer, pack, path),
            };
            let winner = item.source.clone();
            if let Some(previous) = self.verifiers.insert(id.clone(), item) {
                self.overrides
                    .push(CanonOverride::new("verifier", id, previous.source, winner));
            }
            Ok(())
        })?;

        let knowledge_root = root.join("knowledge");
        if knowledge_root.is_dir() {
            for entry in WalkDir::new(&knowledge_root).min_depth(1) {
                let entry = entry.with_context(|| {
                    format!("cannot walk knowledge under {}", knowledge_root.display())
                })?;
                if !entry.file_type().is_file()
                    || entry.path().extension().and_then(|value| value.to_str()) != Some("md")
                {
                    continue;
                }
                let relative = entry
                    .path()
                    .strip_prefix(&knowledge_root)
                    .expect("walk entry is below root")
                    .to_string_lossy()
                    .replace('\\', "/");
                crate::integrity::validate_relative_path(&relative, "knowledge path")?;
                if let Some(existing) = self.knowledge.keys().find(|existing| {
                    existing.eq_ignore_ascii_case(&relative) && *existing != &relative
                }) {
                    bail!(
                        "knowledge paths `{existing}` and `{relative}` collide on a case-insensitive filesystem"
                    );
                }
                let body = fs::read_to_string(entry.path())
                    .with_context(|| format!("cannot read {}", entry.path().display()))?;
                let item = Knowledge {
                    body,
                    source: Source::new(layer, pack, entry.path().to_path_buf()),
                };
                let winner = item.source.clone();
                if let Some(previous) = self.knowledge.insert(relative.clone(), item) {
                    self.overrides.push(CanonOverride::new(
                        "knowledge",
                        relative,
                        previous.source,
                        winner,
                    ));
                }
            }
        }
        Ok(())
    }

    fn load_skills(&mut self, root: &Path, layer: Layer, pack: Option<&str>) -> Result<()> {
        let skills_root = root.join("skills");
        if !skills_root.is_dir() {
            return Ok(());
        }
        let mut seen = BTreeSet::new();
        for entry in fs::read_dir(&skills_root)
            .with_context(|| format!("cannot read {}", skills_root.display()))?
        {
            let entry = entry.with_context(|| format!("cannot read {}", skills_root.display()))?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let directory = entry.path();
            let skill_path = directory.join("SKILL.md");
            if !skill_path.is_file() {
                bail!("skill directory {} has no SKILL.md", directory.display());
            }
            let source = fs::read_to_string(&skill_path)
                .with_context(|| format!("cannot read {}", skill_path.display()))?;
            let (frontmatter, body) = split_frontmatter(&source)
                .with_context(|| format!("invalid frontmatter in {}", skill_path.display()))?;
            let meta: SkillMeta = serde_yaml::from_str(frontmatter)
                .with_context(|| format!("invalid YAML frontmatter in {}", skill_path.display()))?;
            validate_id(&meta.id, "skill")?;
            if entry.file_name().to_string_lossy() != meta.id {
                bail!(
                    "skill directory `{}` must match skill id `{}`",
                    entry.file_name().to_string_lossy(),
                    meta.id
                );
            }
            if !seen.insert(meta.id.clone()) {
                bail!("duplicate skill id `{}` in {}", meta.id, root.display());
            }
            let mut resources = BTreeMap::new();
            let mut portable_resources = BTreeMap::<String, String>::new();
            for resource in WalkDir::new(&directory).min_depth(1) {
                let resource = resource.with_context(|| {
                    format!("cannot walk skill directory {}", directory.display())
                })?;
                if !resource.file_type().is_file() || resource.path() == skill_path {
                    continue;
                }
                let relative = resource
                    .path()
                    .strip_prefix(&directory)
                    .expect("skill resource is below its directory")
                    .to_string_lossy()
                    .replace('\\', "/");
                crate::integrity::validate_relative_path(&relative, "skill resource path")?;
                let normalized = relative.to_ascii_lowercase();
                if let Some(existing) = portable_resources.insert(normalized, relative.clone()) {
                    bail!(
                        "skill `{}` resources `{existing}` and `{relative}` collide on a case-insensitive filesystem",
                        meta.id
                    );
                }
                resources.insert(
                    relative,
                    fs::read(resource.path()).with_context(|| {
                        format!("cannot read skill resource {}", resource.path().display())
                    })?,
                );
            }
            let id = meta.id.clone();
            let item = Skill {
                meta,
                body: body.to_owned(),
                resources,
                source: Source::new(layer, pack, skill_path),
            };
            let winner = item.source.clone();
            if let Some(previous) = self.skills.insert(id.clone(), item) {
                self.overrides
                    .push(CanonOverride::new("skill", id, previous.source, winner));
            }
        }
        Ok(())
    }
}

fn load_documents<T>(
    directory: PathBuf,
    recursive: bool,
    mut insert: impl FnMut(PathBuf, T, String) -> Result<()>,
) -> Result<()>
where
    T: DeserializeOwned + Identified,
{
    if !directory.is_dir() {
        return Ok(());
    }
    let mut portable_paths = BTreeMap::<String, String>::new();
    let mut entries = WalkDir::new(&directory).min_depth(1);
    if !recursive {
        entries = entries.max_depth(1);
    }
    for entry in entries {
        let entry = entry.with_context(|| format!("cannot walk {}", directory.display()))?;
        let path = entry.path();
        if !entry.file_type().is_file()
            || path.extension().and_then(|value| value.to_str()) != Some("md")
        {
            continue;
        }
        let relative = path
            .strip_prefix(&directory)
            .expect("canon document is below its kind directory")
            .to_string_lossy()
            .replace('\\', "/");
        crate::integrity::validate_relative_path(&relative, "canon document path")?;
        let normalized = relative.to_ascii_lowercase();
        if let Some(existing) = portable_paths.insert(normalized, relative.clone()) {
            bail!(
                "canon document paths `{existing}` and `{relative}` collide on a case-insensitive filesystem"
            );
        }
        let source = fs::read_to_string(path)
            .with_context(|| format!("cannot read canon document {}", path.display()))?;
        let (frontmatter, body) = split_frontmatter(&source)
            .with_context(|| format!("invalid frontmatter in {}", path.display()))?;
        let meta: T = serde_yaml::from_str(frontmatter)
            .with_context(|| format!("invalid YAML frontmatter in {}", path.display()))?;
        validate_id(meta.id(), meta.kind())?;
        insert(path.to_path_buf(), meta, body.to_owned())?;
    }
    Ok(())
}

pub fn split_frontmatter(source: &str) -> Result<(&str, &str)> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let (rest, delimiter) = if let Some(rest) = source.strip_prefix("---\r\n") {
        (rest, "\r\n---\r\n")
    } else if let Some(rest) = source.strip_prefix("---\n") {
        (rest, "\n---\n")
    } else {
        bail!("document must start with `---` on its own line");
    };
    let end = rest
        .find(delimiter)
        .context("frontmatter is missing its closing `---`")?;
    let frontmatter = &rest[..end];
    let body = rest[end + delimiter.len()..].trim();
    Ok((frontmatter, body))
}

trait Identified {
    fn id(&self) -> &str;
    fn kind(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleMeta {
    pub id: String,
    #[serde(default)]
    pub description: String,
}

impl Identified for RuleMeta {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &str {
        "rule"
    }
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub meta: RuleMeta,
    pub body: String,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentMeta {
    pub id: String,
    pub description: String,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub access: AgentAccess,
}

impl Identified for AgentMeta {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &str {
        "agent"
    }
}

#[derive(Debug, Clone)]
pub struct Agent {
    pub meta: AgentMeta,
    pub body: String,
    pub source: Source,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AgentAccess {
    #[default]
    Inherit,
    ReadOnly,
    WorkspaceWrite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillMeta {
    pub id: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct Skill {
    pub meta: SkillMeta,
    pub body: String,
    pub resources: BTreeMap<String, Vec<u8>>,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageMeta {
    pub id: String,
    #[serde(default)]
    pub description: String,
}

impl Identified for StageMeta {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &str {
        "stage"
    }
}

#[derive(Debug, Clone)]
pub struct Stage {
    pub meta: StageMeta,
    pub body: String,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineMeta {
    pub id: String,
    pub description: String,
    pub stages: Vec<StageRef>,
}

impl Identified for PipelineMeta {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &str {
        "pipeline"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageRef {
    #[serde(rename = "use")]
    pub use_stage: String,
    #[serde(default)]
    pub gate: Option<String>,
    #[serde(default)]
    pub verifier: Option<String>,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(rename = "independent-review", default)]
    pub independent_review: bool,
}

#[derive(Debug, Clone)]
pub struct Pipeline {
    pub meta: PipelineMeta,
    pub body: String,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyMeta {
    pub id: String,
    pub description: String,
    pub event: PolicyEvent,
    pub mode: PolicyMode,
    pub command: Vec<String>,
    /// Portable tool-name globs. Base compiles these to an anchored regular
    /// expression instead of passing host-specific matcher syntax through.
    #[serde(rename = "match-tools", default)]
    pub match_tools: Vec<String>,
    #[serde(rename = "timeout-seconds", default = "default_policy_timeout")]
    pub timeout_seconds: u64,
    #[serde(rename = "fail-closed", default)]
    pub fail_closed: bool,
}

impl Identified for PolicyMeta {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &str {
        "policy"
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyEvent {
    SessionStart,
    PreToolUse,
    PostToolUse,
    SessionEnd,
}

impl PolicyEvent {
    pub fn claude_name(self) -> &'static str {
        match self {
            Self::SessionStart => "SessionStart",
            Self::PreToolUse => "PreToolUse",
            Self::PostToolUse => "PostToolUse",
            Self::SessionEnd => "SessionEnd",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyMode {
    Context,
    Guard,
    Observe,
}

#[derive(Debug, Clone)]
pub struct Policy {
    pub meta: PolicyMeta,
    pub body: String,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifierMeta {
    pub id: String,
    pub description: String,
    pub checks: Vec<VerifierCheck>,
}

impl Identified for VerifierMeta {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &str {
        "verifier"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifierCheck {
    pub id: String,
    pub run: Vec<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(rename = "timeout-seconds", default = "default_verifier_timeout")]
    pub timeout_seconds: u64,
    /// Output is hashed and counted by default, but retained only after an explicit opt-in.
    #[serde(rename = "retain-output", default)]
    pub retain_output: bool,
}

#[derive(Debug, Clone)]
pub struct Verifier {
    pub meta: VerifierMeta,
    pub body: String,
    pub source: Source,
}

#[derive(Debug, Clone)]
pub struct Knowledge {
    pub body: String,
    pub source: Source,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Global,
    Pack,
    Project,
}

#[derive(Debug, Clone)]
pub struct Source {
    pub layer: Layer,
    pub pack: Option<String>,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct CanonOverride {
    pub kind: &'static str,
    pub id: String,
    pub replaced: Source,
    pub winner: Source,
}

impl CanonOverride {
    fn new(kind: &'static str, id: String, replaced: Source, winner: Source) -> Self {
        Self {
            kind,
            id,
            replaced,
            winner,
        }
    }
}

impl Source {
    fn new(layer: Layer, pack: Option<&str>, path: PathBuf) -> Self {
        Self {
            layer,
            pack: pack.map(str::to_owned),
            path,
        }
    }

    pub fn repo_resident(&self) -> bool {
        self.layer != Layer::Global
    }
}

fn validate_policy(meta: &PolicyMeta) -> Result<()> {
    if meta.description.trim().is_empty() {
        bail!("policy `{}` description cannot be empty", meta.id);
    }
    if meta.command.is_empty() || meta.command.iter().any(|part| part.is_empty()) {
        bail!("policy `{}` command must be non-empty argv", meta.id);
    }
    if !(1..=55).contains(&meta.timeout_seconds) {
        bail!("policy `{}` timeout-seconds must be 1..=55", meta.id);
    }
    if meta.mode == PolicyMode::Context && meta.event != PolicyEvent::SessionStart {
        bail!("context policy `{}` must use event session-start", meta.id);
    }
    if meta.mode == PolicyMode::Guard && meta.event != PolicyEvent::PreToolUse {
        bail!("guard policy `{}` must use event pre-tool-use", meta.id);
    }
    if meta.fail_closed && meta.mode != PolicyMode::Guard {
        bail!(
            "policy `{}` fail-closed applies only to guard mode",
            meta.id
        );
    }
    if !meta.match_tools.is_empty() {
        if !matches!(
            meta.event,
            PolicyEvent::PreToolUse | PolicyEvent::PostToolUse
        ) {
            bail!(
                "policy `{}` match-tools applies only to pre-tool-use or post-tool-use events",
                meta.id
            );
        }
        let mut tools = BTreeSet::new();
        for tool in &meta.match_tools {
            if tool.trim().is_empty() || tool.trim() != tool || tool.chars().any(char::is_control) {
                bail!(
                    "policy `{}` match-tools entries must be non-empty, trimmed, printable globs",
                    meta.id
                );
            }
            if !tools.insert(tool) {
                bail!("policy `{}` repeats match-tool `{tool}`", meta.id);
            }
        }
    }
    Ok(())
}

fn validate_agent(meta: &AgentMeta) -> Result<()> {
    let mut tools = BTreeSet::new();
    for tool in &meta.tools {
        if tool.trim().is_empty()
            || tool.trim() != tool
            || tool.chars().any(char::is_control)
            || tool.contains(',')
        {
            bail!(
                "agent `{}` tools entries must be non-empty, trimmed, printable single tool names without commas",
                meta.id
            );
        }
        if !tools.insert(tool) {
            bail!("agent `{}` repeats tool `{tool}`", meta.id);
        }
    }
    Ok(())
}

fn validate_verifier(meta: &VerifierMeta) -> Result<()> {
    if meta.description.trim().is_empty() {
        bail!("verifier `{}` description cannot be empty", meta.id);
    }
    if meta.checks.is_empty() {
        bail!("verifier `{}` has no checks", meta.id);
    }
    let mut ids = BTreeSet::new();
    for check in &meta.checks {
        validate_id(&check.id, "verifier check")?;
        if !ids.insert(check.id.as_str()) {
            bail!("verifier `{}` repeats check `{}`", meta.id, check.id);
        }
        if check.run.is_empty() || check.run.iter().any(|part| part.is_empty()) {
            bail!(
                "verifier `{}` check `{}` run must be non-empty argv",
                meta.id,
                check.id
            );
        }
        if !(1..=3600).contains(&check.timeout_seconds) {
            bail!(
                "verifier `{}` check `{}` timeout-seconds must be 1..=3600",
                meta.id,
                check.id
            );
        }
        if let Some(cwd) = &check.cwd {
            crate::integrity::validate_relative_path(cwd, "verifier check cwd")?;
        }
    }
    Ok(())
}

fn default_policy_timeout() -> u64 {
    10
}

fn default_verifier_timeout() -> u64 {
    300
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn frontmatter_is_split() {
        let (yaml, body) = split_frontmatter("---\nid: test\n---\n\nDo the thing.\n").unwrap();
        assert_eq!(yaml, "id: test");
        assert_eq!(body, "Do the thing.");
    }

    #[test]
    fn missing_frontmatter_is_rejected() {
        assert!(split_frontmatter("# Hello").is_err());
    }

    #[test]
    fn windows_frontmatter_is_split() {
        let (yaml, body) = split_frontmatter("---\r\nid: test\r\n---\r\nBody\r\n").unwrap();
        assert_eq!(yaml, "id: test");
        assert_eq!(body, "Body");
    }

    #[test]
    fn duplicate_ids_in_one_layer_are_rejected() {
        let project = tempfile::tempdir().unwrap();
        let rules = project.path().join(".base/canon/rules");
        fs::create_dir_all(&rules).unwrap();
        fs::write(rules.join("one.md"), "---\nid: duplicate\n---\nOne\n").unwrap();
        fs::write(rules.join("two.md"), "---\nid: duplicate\n---\nTwo\n").unwrap();
        let global = tempfile::tempdir().unwrap();
        let error = Canon::load(global.path(), project.path(), &Config::default()).unwrap_err();
        assert!(error.to_string().contains("duplicate rule id"));
    }
}
