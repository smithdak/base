mod approve;
mod check;
mod hook;
mod init;
mod log;
mod sync;
mod work;

use std::env;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Serialize;

use crate::cli::{Cli, Command};
use crate::{base_home, find_project_root};

pub fn run(cli: Cli) -> Result<()> {
    if let Command::Hook(args) = cli.command {
        return hook::run(args);
    }

    let start = selected_directory(cli.directory.as_deref())?;
    match cli.command {
        Command::Init(args) => init::run(&start, args, cli.json),
        Command::Sync(args) => sync::run(&find_project_root(&start)?, args, cli.json),
        Command::Check => check::run(&find_project_root(&start)?, cli.json),
        Command::Work(args) => work::run(&find_project_root(&start)?, args, cli.json),
        Command::Log(args) => log::run(&find_project_root(&start)?, args, cli.json),
        Command::Approve(args) => approve::run(&find_project_root(&start)?, args, cli.json),
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
