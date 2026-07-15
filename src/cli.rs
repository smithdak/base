use std::fmt;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum WorkStatus {
    Todo,
    Doing,
    Review,
    Done,
}

impl WorkStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::Doing => "doing",
            Self::Review => "review",
            Self::Done => "done",
        }
    }
}

impl fmt::Display for WorkStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum WorkVerdict {
    #[default]
    #[value(skip)]
    Pending,
    Pass,
    Fail,
}

impl WorkVerdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Pass => "pass",
            Self::Fail => "fail",
        }
    }
}

impl fmt::Display for WorkVerdict {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Parser)]
#[command(name = "base", version, about)]
pub struct Cli {
    /// Emit machine-readable JSON.
    #[arg(long, global = true)]
    pub json: bool,

    /// Start discovery from this directory.
    #[arg(long, global = true, value_name = "PATH")]
    pub directory: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Scaffold a global canon or project overlay.
    Init(InitArgs),
    /// Compile canon into active harness surfaces.
    Sync(SyncArgs),
    /// Validate canon and report gate enforcement fidelity.
    Check,
    /// Manage project work items.
    Work(WorkArgs),
    /// Inspect run history or one run folder.
    Log(LogArgs),
    /// Internal harness-hook entrypoint.
    #[command(name = "__hook", hide = true)]
    Hook(HookArgs),
}

#[derive(Debug, Args)]
#[group(required = false, multiple = false)]
pub struct InitArgs {
    /// Initialize the user-wide canon at BASE_HOME (normally ~/.base).
    #[arg(long, group = "scope")]
    pub global: bool,

    /// Initialize a project in the selected directory even outside git.
    #[arg(long, group = "scope")]
    pub project: bool,

    /// Replace base-owned scaffold files that already exist.
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct SyncArgs {
    /// Validate and detect missing, stale, or modified output without writing.
    #[arg(long)]
    pub check: bool,

    /// Replace hand-modified generated output.
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct WorkArgs {
    #[command(subcommand)]
    pub command: WorkCommand,
}

#[derive(Debug, Subcommand)]
pub enum WorkCommand {
    /// List work items.
    List {
        /// Filter by status.
        #[arg(long)]
        status: Option<WorkStatus>,
    },
    /// Create a work item.
    New {
        /// Short work-item title.
        title: String,
        /// Comma-separated tags.
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
        /// Acceptance criterion. Repeat for multiple criteria.
        #[arg(long, value_name = "TEXT")]
        criterion: Vec<String>,
    },
    /// Print one work item by ID or folder name.
    Show { id: String },
    /// Move one work item to a workflow status.
    Move {
        /// Work-item ID or folder name.
        id: String,
        /// Target workflow status.
        status: WorkStatus,
        /// Human verdict required when moving to done.
        #[arg(long)]
        verdict: Option<WorkVerdict>,
    },
    /// Render the kanban work board.
    Board,
}

#[derive(Debug, Args)]
pub struct LogArgs {
    /// Run slug. Omit to list history.
    pub slug: Option<String>,
}

#[derive(Debug, Args)]
pub struct HookArgs {
    #[command(subcommand)]
    pub command: HookCommand,
}

#[derive(Debug, Subcommand)]
pub enum HookCommand {
    #[command(name = "claude-pre-tool")]
    ClaudePreTool {
        #[arg(long, default_value = "main")]
        default_branch: String,
    },
}
