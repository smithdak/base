mod adopt;
mod approve;
mod check;
mod hook;
mod init;
mod log;
mod state;
mod sync;
mod verify;
mod work;

use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use serde::Serialize;

use crate::cli::{Cli, Command};
use crate::lock::{LockMode, RepositoryLock};
use crate::{base_home, find_project_root};

pub fn run(cli: Cli) -> Result<()> {
    if let Command::Hook(args) = cli.command {
        return hook::run(args);
    }

    let start = selected_directory(cli.directory.as_deref())?;
    match cli.command {
        Command::Init(args) => init::run(&start, args, cli.json),
        Command::Sync(args) => {
            let root = find_project_root(&start)?;
            let mode = if args.check {
                LockMode::Shared
            } else {
                LockMode::Exclusive
            };
            let _lock = RepositoryLock::project(&root, mode)?;
            sync::run(&root, args, cli.json)
        }
        Command::Check => {
            let root = find_project_root(&start)?;
            let _lock = RepositoryLock::project(&root, LockMode::Shared)?;
            check::run(&root, cli.json)
        }
        Command::Adopt(args) => {
            let root = find_project_root(&start)?;
            let _lock = RepositoryLock::project(&root, LockMode::Exclusive)?;
            adopt::run(&root, args, cli.json)
        }
        Command::Work(args) => {
            let root = find_project_root(&start)?;
            let mode = match &args.command {
                crate::cli::WorkCommand::New { .. } | crate::cli::WorkCommand::Move { .. } => {
                    LockMode::Exclusive
                }
                _ => LockMode::Shared,
            };
            let _lock = RepositoryLock::project(&root, mode)?;
            work::run(&root, args, cli.json)
        }
        Command::Log(args) => {
            let root = find_project_root(&start)?;
            let _lock = RepositoryLock::project(&root, LockMode::Shared)?;
            log::run(&root, args, cli.json)
        }
        Command::Approve(args) => {
            let root = find_project_root(&start)?;
            let _lock = RepositoryLock::project(&root, LockMode::Exclusive)?;
            approve::run(&root, args, cli.json)
        }
        Command::Verify(args) => {
            let root = find_project_root(&start)?;
            let _lock = RepositoryLock::project(&root, LockMode::Shared)?;
            verify::run(&root, args, cli.json)
        }
        Command::State(args) => {
            let root = find_project_root(&start)?;
            let mode = match &args.command {
                crate::cli::StateCommand::Set { .. } | crate::cli::StateCommand::Clear => {
                    LockMode::Exclusive
                }
                _ => LockMode::Shared,
            };
            let _lock = RepositoryLock::project(&root, mode)?;
            state::run(&root, args, cli.json)
        }
        Command::Hook(_) => unreachable!("hook handled before project discovery"),
    }
}

fn selected_directory(value: Option<&Path>) -> Result<PathBuf> {
    match value {
        Some(path) => Ok(path.canonicalize()?),
        None => Ok(env::current_dir()?),
    }
}

fn load_project(project_root: &Path) -> Result<(crate::config::Config, crate::canon::Canon)> {
    let config = crate::config::Config::load(project_root)?;
    let canon = crate::canon::Canon::load(&base_home()?, project_root, &config)?;
    Ok((config, canon))
}

fn print_json(value: &impl Serialize) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn validate_run_slug(slug: &str) -> Result<()> {
    let bytes = slug.as_bytes();
    let valid = !bytes.is_empty()
        && bytes.len() <= 128
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
        && bytes.first().is_some_and(u8::is_ascii_alphanumeric)
        && bytes.last().is_some_and(u8::is_ascii_alphanumeric)
        && !slug.contains("--");
    if !valid {
        bail!(
            "invalid run slug `{slug}`; use 1-128 lowercase ASCII letters, digits, and single hyphens"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_slugs_are_portable_kebab_names() {
        for valid in ["demo", "2026-07-20-operating-model-core", "run-2"] {
            assert!(validate_run_slug(valid).is_ok(), "{valid}");
        }
        for invalid in [
            "",
            "Demo",
            "two words",
            "line\nbreak",
            "a--b",
            "a/b",
            "nul?",
        ] {
            assert!(validate_run_slug(invalid).is_err(), "{invalid:?}");
        }
    }
}
