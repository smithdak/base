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
    pub stages: BTreeMap<String, Stage>,
    pub pipelines: BTreeMap<String, Pipeline>,
    pub knowledge: BTreeMap<String, Knowledge>,
}

impl Canon {
    pub fn load(global_root: &Path, project_root: &Path, config: &Config) -> Result<Self> {
        let mut canon = Self {
            rules: BTreeMap::new(),
            agents: BTreeMap::new(),
            stages: BTreeMap::new(),
            pipelines: BTreeMap::new(),
            knowledge: BTreeMap::new(),
        };

        let global_canon = global_root.join("canon");
        if global_canon.is_dir() {
            canon.load_layer(&global_canon, Layer::Global)?;
        }
        let project_canon = project_root.join(".base/canon");
        if project_canon.is_dir() {
            canon.load_layer(&project_canon, Layer::Project)?;
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
                if pipeline.source.layer == Layer::Project && stage.source.layer == Layer::Global {
                    bail!(
                        "project pipeline `{}` references global-only stage `{}`; copy the stage into `.base/canon/pipelines/stages/` to adopt it",
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
            }
            let final_stage = pipeline.meta.stages.last().expect("checked non-empty");
            if final_stage.use_stage != "record" {
                bail!(
                    "pipeline `{}` must end with the `record` stage so every exit is logged",
                    pipeline.meta.id
                );
            }
        }
        Ok(())
    }

    fn load_layer(&mut self, root: &Path, layer: Layer) -> Result<()> {
        let mut seen_rules = BTreeSet::new();
        load_documents::<RuleMeta>(root.join("rules"), false, |path, meta, body| {
            if !seen_rules.insert(meta.id.clone()) {
                bail!("duplicate rule id `{}` in {}", meta.id, root.display());
            }
            self.rules.insert(
                meta.id.clone(),
                Rule {
                    meta,
                    body,
                    source: Source::new(layer, path),
                },
            );
            Ok(())
        })?;

        let mut seen_agents = BTreeSet::new();
        load_documents::<AgentMeta>(root.join("agents"), false, |path, meta, body| {
            if !seen_agents.insert(meta.id.clone()) {
                bail!("duplicate agent id `{}` in {}", meta.id, root.display());
            }
            self.agents.insert(
                meta.id.clone(),
                Agent {
                    meta,
                    body,
                    source: Source::new(layer, path),
                },
            );
            Ok(())
        })?;

        let mut seen_stages = BTreeSet::new();
        load_documents::<StageMeta>(root.join("pipelines/stages"), false, |path, meta, body| {
            if !seen_stages.insert(meta.id.clone()) {
                bail!("duplicate stage id `{}` in {}", meta.id, root.display());
            }
            self.stages.insert(
                meta.id.clone(),
                Stage {
                    meta,
                    body,
                    source: Source::new(layer, path),
                },
            );
            Ok(())
        })?;

        let mut seen_pipelines = BTreeSet::new();
        load_documents::<PipelineMeta>(root.join("pipelines"), false, |path, meta, body| {
            if !seen_pipelines.insert(meta.id.clone()) {
                bail!("duplicate pipeline id `{}` in {}", meta.id, root.display());
            }
            self.pipelines.insert(
                meta.id.clone(),
                Pipeline {
                    meta,
                    body,
                    source: Source::new(layer, path),
                },
            );
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
                let body = fs::read_to_string(entry.path())
                    .with_context(|| format!("cannot read {}", entry.path().display()))?;
                self.knowledge.insert(
                    relative,
                    Knowledge {
                        body,
                        source: Source::new(layer, entry.path().to_path_buf()),
                    },
                );
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
pub struct AgentMeta {
    pub id: String,
    pub description: String,
    #[serde(default)]
    pub tools: Vec<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct StageRef {
    #[serde(rename = "use")]
    pub use_stage: String,
    #[serde(default)]
    pub gate: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Pipeline {
    pub meta: PipelineMeta,
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
    Project,
}

#[derive(Debug, Clone)]
pub struct Source {
    pub layer: Layer,
    pub path: PathBuf,
}

impl Source {
    fn new(layer: Layer, path: PathBuf) -> Self {
        Self { layer, path }
    }
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
