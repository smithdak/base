pub mod canon;
pub mod cli;
pub mod commands;
pub mod config;
pub mod integrity;
pub mod lock;
pub mod pack;
pub(crate) mod process;
pub mod render;
pub mod templates;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

pub fn find_project_root(start: &Path) -> Result<PathBuf> {
    let start = start
        .canonicalize()
        .with_context(|| format!("cannot resolve {}", start.display()))?;

    for directory in start.ancestors() {
        if directory.join(".base/base.toml").is_file() || directory.join(".git").exists() {
            return Ok(directory.to_path_buf());
        }
    }

    bail!("not inside a base project or git repository (run `base init --project` first)")
}

pub fn base_home() -> Result<PathBuf> {
    if let Some(value) = std::env::var_os("BASE_HOME") {
        return Ok(PathBuf::from(value));
    }
    dirs::home_dir()
        .map(|home| home.join(".base"))
        .context("cannot determine the home directory; set BASE_HOME")
}
