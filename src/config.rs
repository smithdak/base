use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default = "default_targets")]
    pub targets: Vec<Target>,
    #[serde(default = "default_branch")]
    pub default_branch: String,
    #[serde(default = "default_gates")]
    pub gates: Vec<Gate>,
    #[serde(default)]
    pub generated: BTreeMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: default_version(),
            targets: default_targets(),
            default_branch: default_branch(),
            gates: default_gates(),
            generated: BTreeMap::new(),
        }
    }
}

impl Config {
    pub fn path(project_root: &Path) -> PathBuf {
        project_root.join(".base/base.toml")
    }

    pub fn load(project_root: &Path) -> Result<Self> {
        let path = Self::path(project_root);
        let source = fs::read_to_string(&path)
            .with_context(|| format!("cannot read {}; run `base init`", path.display()))?;
        let config: Self = toml::from_str(&source)
            .with_context(|| format!("invalid TOML in {}", path.display()))?;
        config.validate()?;
        Ok(config)
    }

    pub fn save(&self, project_root: &Path) -> Result<()> {
        let path = Self::path(project_root);
        let source = toml::to_string_pretty(self).context("cannot serialize base config")?;
        fs::write(&path, source).with_context(|| format!("cannot write {}", path.display()))
    }

    pub fn validate(&self) -> Result<()> {
        if self.version != 1 {
            bail!(
                "unsupported base config version {}; expected 1",
                self.version
            );
        }
        if self.targets.is_empty() {
            bail!("base config must enable at least one target");
        }
        let mut target_set = BTreeSet::new();
        for target in &self.targets {
            if !target_set.insert(target) {
                bail!("target `{target}` is listed more than once");
            }
        }
        if self.default_branch.trim().is_empty() {
            bail!("default_branch cannot be empty");
        }
        let mut gate_ids = BTreeSet::new();
        for gate in &self.gates {
            validate_id(&gate.id, "gate")?;
            if !gate_ids.insert(gate.id.as_str()) {
                bail!("gate `{}` is declared more than once", gate.id);
            }
            if let Some(path) = &gate.satisfied_by {
                if gate.kind != GateKind::StageApproval {
                    bail!(
                        "gate `{}`: satisfied-by applies only to stage-approval gates",
                        gate.id
                    );
                }
                let clean = path.trim();
                // has_root catches `/foo` on Windows too (where it is not
                // `is_absolute`); `:` blocks drive designators either way.
                if clean.is_empty()
                    || Path::new(clean).has_root()
                    || clean.contains(':')
                    || clean.split(['/', '\\']).any(|part| part == "..")
                {
                    bail!(
                        "gate `{}`: satisfied-by must be a relative path inside the run folder",
                        gate.id
                    );
                }
            }
        }
        Ok(())
    }

    pub fn gate(&self, id: &str) -> Option<&Gate> {
        self.gates.iter().find(|gate| gate.id == id)
    }
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum,
)]
#[serde(rename_all = "lowercase")]
pub enum Target {
    Claude,
    Codex,
    Copilot,
}

impl fmt::Display for Target {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Copilot => "copilot",
        })
    }
}

impl FromStr for Target {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "claude" => Ok(Self::Claude),
            "codex" => Ok(Self::Codex),
            "copilot" => Ok(Self::Copilot),
            _ => bail!("unknown target `{value}`"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Gate {
    pub id: String,
    pub kind: GateKind,
    pub description: String,
    /// Run-folder-relative path of the artifact that satisfies this stage gate.
    #[serde(
        rename = "satisfied-by",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub satisfied_by: Option<String>,
}

impl Gate {
    /// The response artifact path, relative to the run folder.
    pub fn approval_path(&self) -> String {
        self.satisfied_by
            .clone()
            .unwrap_or_else(|| format!("approvals/{}.md", self.id))
    }

    /// The pending-request marker derived from the response path.
    pub fn request_path(&self) -> String {
        format!("{}.request", self.approval_path())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum GateKind {
    StageApproval,
    StandingDenial,
}

impl fmt::Display for GateKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::StageApproval => "stage-approval",
            Self::StandingDenial => "standing-denial",
        })
    }
}

pub fn validate_id(id: &str, kind: &str) -> Result<()> {
    let valid = !id.is_empty()
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && id.as_bytes().first().is_some_and(u8::is_ascii_lowercase);
    if !valid {
        bail!(
            "invalid {kind} id `{id}`; use lowercase letters, digits, and hyphens, starting with a letter"
        );
    }
    Ok(())
}

fn default_version() -> u32 {
    1
}

fn default_targets() -> Vec<Target> {
    vec![Target::Claude, Target::Codex, Target::Copilot]
}

fn default_branch() -> String {
    "main".to_owned()
}

fn default_gates() -> Vec<Gate> {
    vec![
        Gate {
            id: "plan-approval".to_owned(),
            kind: GateKind::StageApproval,
            description: "Do not execute until the user explicitly approves the written plan."
                .to_owned(),
            satisfied_by: Some("approvals/plan-approval.md".to_owned()),
        },
        Gate {
            id: "never-push-default-branch".to_owned(),
            kind: GateKind::StandingDenial,
            description: "Never push directly to the repository default branch.".to_owned(),
            satisfied_by: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_round_trips() {
        let source = toml::to_string_pretty(&Config::default()).unwrap();
        let parsed: Config = toml::from_str(&source).unwrap();
        assert_eq!(parsed, Config::default());
    }

    #[test]
    fn identifiers_are_conservative() {
        assert!(validate_id("plan-approval", "gate").is_ok());
        assert!(validate_id("Plan", "gate").is_err());
        assert!(validate_id("2plan", "gate").is_err());
    }

    #[test]
    fn satisfied_by_is_constrained_to_stage_gates_and_safe_paths() {
        let mut config = Config::default();
        config.gates[1].satisfied_by = Some("approvals/deny.md".to_owned());
        assert!(config.validate().is_err(), "standing denial cannot declare");
        let mut config = Config::default();
        config.gates[0].satisfied_by = Some("../outside.md".to_owned());
        assert!(config.validate().is_err(), "path escapes the run folder");
        let mut config = Config::default();
        config.gates[0].satisfied_by = Some("/absolute.md".to_owned());
        assert!(config.validate().is_err(), "absolute path rejected");
        assert!(Config::default().validate().is_ok());
    }

    #[test]
    fn approval_paths_derive_mechanically() {
        let gate = Config::default().gate("plan-approval").unwrap().clone();
        assert_eq!(gate.approval_path(), "approvals/plan-approval.md");
        assert_eq!(gate.request_path(), "approvals/plan-approval.md.request");
        let bare = Gate {
            satisfied_by: None,
            ..gate
        };
        assert_eq!(bare.approval_path(), "approvals/plan-approval.md");
    }
}
