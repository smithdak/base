use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default = "default_version")]
    pub version: u32,
    /// Semver range for the Base compiler/runtime that owns this repository contract.
    #[serde(
        rename = "requires-base",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub requires_base: Option<String>,
    #[serde(default = "default_targets")]
    pub targets: Vec<Target>,
    #[serde(default = "default_branch")]
    pub default_branch: String,
    #[serde(default = "default_gates")]
    pub gates: Vec<Gate>,
    /// Ordered, repository-vendored composition layers. Later packs win by canonical ID.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packs: Vec<PackRecord>,
    #[serde(default)]
    pub generated: BTreeMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: default_version(),
            requires_base: Some(default_base_requirement()),
            targets: default_targets(),
            default_branch: default_branch(),
            gates: default_gates(),
            packs: Vec::new(),
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
        let parent = path.parent().expect("base config has a parent directory");
        let mut temporary = tempfile::NamedTempFile::new_in(parent).with_context(|| {
            format!(
                "cannot create a temporary config under {}",
                parent.display()
            )
        })?;
        temporary
            .write_all(source.as_bytes())
            .context("cannot write temporary base config")?;
        temporary
            .as_file()
            .sync_all()
            .context("cannot sync temporary base config")?;
        temporary
            .persist(&path)
            .map(|_| ())
            .map_err(|error| error.error)
            .with_context(|| format!("cannot atomically replace {}", path.display()))
    }

    pub fn validate(&self) -> Result<()> {
        if self.version != 2 {
            bail!(
                "unsupported base config version {}; expected 2",
                self.version
            );
        }
        if let Some(requirement) = &self.requires_base {
            let requirement = semver::VersionReq::parse(requirement)
                .context("requires-base must be a valid semantic-version requirement")?;
            let runtime = semver::Version::parse(env!("CARGO_PKG_VERSION"))
                .expect("Cargo package version is valid semver");
            if !requirement.matches(&runtime) {
                bail!(
                    "project requires Base `{requirement}`, but this runtime is {runtime}; install a compatible Base release"
                );
            }
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
        validate_branch_name(&self.default_branch)?;
        let mut gate_ids = BTreeSet::new();
        let mut approval_paths = BTreeMap::<String, String>::new();
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
                crate::integrity::validate_relative_path(
                    path,
                    &format!("gate `{}` satisfied-by", gate.id),
                )?;
                let normalized = path.to_ascii_lowercase();
                if let Some(existing) = approval_paths.insert(normalized, gate.id.clone()) {
                    bail!(
                        "gates `{existing}` and `{}` declare the same satisfied-by path `{path}`",
                        gate.id
                    );
                }
            }
        }
        let mut pack_ids = BTreeSet::new();
        for pack in &self.packs {
            validate_id(&pack.id, "pack")?;
            if !pack_ids.insert(pack.id.as_str()) {
                bail!("pack `{}` is listed more than once", pack.id);
            }
            semver::Version::parse(&pack.version)
                .with_context(|| format!("pack `{}` has invalid semantic version", pack.id))?;
            if pack.files.is_empty() {
                bail!("pack `{}` has no recorded files", pack.id);
            }
            for (path, hash) in &pack.files {
                crate::integrity::validate_relative_path(path, "pack file path")?;
                if !crate::integrity::is_sha256(hash) {
                    bail!("pack `{}` has invalid SHA-256 for `{path}`", pack.id);
                }
            }
        }
        Ok(())
    }

    pub fn gate(&self, id: &str) -> Option<&Gate> {
        self.gates.iter().find(|gate| gate.id == id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PackRecord {
    pub id: String,
    pub version: String,
    pub description: String,
    pub files: BTreeMap<String, String>,
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
#[serde(deny_unknown_fields)]
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
    if !crate::integrity::portable_component(id) {
        bail!("invalid {kind} id `{id}`; the name is not portable across Windows and Unix");
    }
    Ok(())
}

fn validate_branch_name(branch: &str) -> Result<()> {
    let invalid = branch.is_empty()
        || branch != branch.trim()
        || branch == "@"
        || branch.starts_with('-')
        || branch.starts_with('/')
        || branch.ends_with('/')
        || branch.ends_with('.')
        || branch.contains("..")
        || branch.contains("@{")
        || branch.contains("//")
        || branch
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
        || branch.contains(['~', '^', ':', '?', '*', '[', '\\'])
        || branch.split('/').any(|component| {
            component.is_empty() || component.starts_with('.') || component.ends_with(".lock")
        });
    if invalid {
        bail!(
            "invalid default_branch `{branch}`; use a valid Git branch name without whitespace, control characters, ref metacharacters, or traversal components"
        );
    }
    Ok(())
}

fn default_version() -> u32 {
    2
}

fn default_base_requirement() -> String {
    ">=0.2.0, <0.3.0".to_owned()
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
    fn v1_runtime_contract_is_rejected_before_mutation() {
        let config = Config {
            version: 1,
            ..Config::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn identifiers_are_conservative() {
        assert!(validate_id("plan-approval", "gate").is_ok());
        assert!(validate_id("Plan", "gate").is_err());
        assert!(validate_id("2plan", "gate").is_err());
        assert!(validate_id("con", "gate").is_err());
    }

    #[test]
    fn default_branch_is_safe_for_native_renderers() {
        for valid in ["main", "release/2026-07", "feature_base"] {
            let config = Config {
                default_branch: valid.to_owned(),
                ..Config::default()
            };
            assert!(config.validate().is_ok(), "{valid}");
        }
        for invalid in [
            "",
            " main",
            "main ",
            "-main",
            ".main",
            "a..b",
            "a@{b",
            "a b",
            "a~b",
            "a:b",
            "a\\b",
            "topic.lock",
            "topic/.hidden",
        ] {
            let config = Config {
                default_branch: invalid.to_owned(),
                ..Config::default()
            };
            assert!(config.validate().is_err(), "{invalid}");
        }
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
        let mut config = Config::default();
        config.gates[0].satisfied_by = Some("approvals\\decision.md".to_owned());
        assert!(
            config.validate().is_err(),
            "non-portable separator rejected"
        );
        assert!(Config::default().validate().is_ok());
    }

    #[test]
    fn stage_gates_cannot_share_an_approval_artifact() {
        let mut config = Config::default();
        config.gates.push(Gate {
            id: "security-review".to_owned(),
            kind: GateKind::StageApproval,
            description: "Review security".to_owned(),
            satisfied_by: Some("APPROVALS/PLAN-APPROVAL.MD".to_owned()),
        });
        assert!(config.validate().is_err());
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
