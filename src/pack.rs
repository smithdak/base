use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::canon::split_frontmatter;
use crate::config::{PackRecord, validate_id};
use crate::integrity::digest;

const CANON_ROOTS: &[&str] = &[
    "agents",
    "knowledge",
    "pipelines",
    "policies",
    "rules",
    "skills",
    "verifiers",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PackManifest {
    pub id: String,
    pub version: String,
    pub description: String,
}

pub fn installed_root(project_root: &Path, id: &str) -> PathBuf {
    project_root.join(".base").join("packs").join(id)
}

pub fn library_root(base_home: &Path, id: &str) -> PathBuf {
    base_home.join("canon").join("packs").join(id)
}

pub fn read_manifest(root: &Path) -> Result<PackManifest> {
    let path = root.join("pack.md");
    let source = fs::read_to_string(&path)
        .with_context(|| format!("cannot read pack manifest {}", path.display()))?;
    let (frontmatter, _) = split_frontmatter(&source)
        .with_context(|| format!("invalid pack manifest {}", path.display()))?;
    let manifest: PackManifest = serde_yaml::from_str(frontmatter)
        .with_context(|| format!("invalid pack manifest YAML in {}", path.display()))?;
    validate_id(&manifest.id, "pack")?;
    semver::Version::parse(&manifest.version)
        .with_context(|| format!("pack `{}` has invalid semantic version", manifest.id))?;
    if manifest.description.trim().is_empty() {
        bail!("pack `{}` description cannot be empty", manifest.id);
    }
    Ok(manifest)
}

pub fn build_record(root: &Path) -> Result<PackRecord> {
    let manifest = read_manifest(root)?;
    let files = collect_hashes(root)?;
    Ok(PackRecord {
        id: manifest.id,
        version: manifest.version,
        description: manifest.description,
        files,
    })
}

pub fn collect_files(root: &Path) -> Result<BTreeMap<String, Vec<u8>>> {
    if !root.is_dir() {
        bail!("pack directory does not exist: {}", root.display());
    }
    let mut files = BTreeMap::new();
    let mut portable_paths = BTreeMap::<String, String>::new();
    for entry in WalkDir::new(root).min_depth(1) {
        let entry = entry.with_context(|| format!("cannot walk pack {}", root.display()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(root)
            .expect("pack entry is below root")
            .to_string_lossy()
            .replace('\\', "/");
        validate_pack_path(&relative)?;
        let normalized = relative.to_ascii_lowercase();
        if let Some(existing) = portable_paths.insert(normalized, relative.clone()) {
            bail!(
                "pack contains case-colliding paths `{existing}` and `{relative}`; paths must be portable to case-insensitive filesystems"
            );
        }
        let content = fs::read(entry.path())
            .with_context(|| format!("cannot read pack file {}", entry.path().display()))?;
        files.insert(relative, content);
    }
    if !files.contains_key("pack.md") {
        bail!("pack at {} has no pack.md manifest", root.display());
    }
    if files.len() == 1 {
        bail!("pack at {} has no canonical content", root.display());
    }
    Ok(files)
}

pub fn collect_hashes(root: &Path) -> Result<BTreeMap<String, String>> {
    Ok(collect_files(root)?
        .into_iter()
        .map(|(path, content)| (path, digest(&content)))
        .collect())
}

pub fn verify_installed(project_root: &Path, record: &PackRecord) -> Result<()> {
    let root = installed_root(project_root, &record.id);
    let manifest = read_manifest(&root)?;
    if manifest.id != record.id || manifest.version != record.version {
        bail!(
            "adopted pack `{}` manifest differs from .base/base.toml (manifest {} {}, recorded {} {}); re-adopt or restore the recorded files",
            record.id,
            manifest.id,
            manifest.version,
            record.id,
            record.version
        );
    }
    let actual = collect_hashes(&root)?;
    if actual == record.files {
        return Ok(());
    }
    let mut drift = Vec::new();
    for (path, expected) in &record.files {
        match actual.get(path) {
            None => drift.push(format!("missing {path}")),
            Some(found) if found != expected => drift.push(format!("changed {path}")),
            Some(_) => {}
        }
    }
    for path in actual.keys() {
        if !record.files.contains_key(path) {
            drift.push(format!("untracked {path}"));
        }
    }
    bail!(
        "adopted pack `{}` was modified locally: {}; put project changes in .base/canon/ overrides or restore the pack",
        record.id,
        drift.join(", ")
    )
}

fn validate_pack_path(relative: &str) -> Result<()> {
    crate::integrity::validate_relative_path(relative, "pack file path")?;
    if relative == "pack.md" {
        return Ok(());
    }
    let top = relative.split('/').next().unwrap_or_default();
    if !CANON_ROOTS.contains(&top) {
        bail!(
            "unsupported pack path `{relative}`; top-level content must be pack.md or one of {}",
            CANON_ROOTS.join(", ")
        );
    }
    let extension = Path::new(relative)
        .extension()
        .and_then(|value| value.to_str());
    if top != "skills" && extension != Some("md") {
        bail!("pack canonical file `{relative}` must be Markdown");
    }
    Ok(())
}
