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
    /// Copy a global-library pack into the project canon.
    Adopt(AdoptArgs),
    /// Inverse-read another system's harness surfaces into a portable inventory.
    Ingest(IngestArgs),
    /// Scaffold or validate a canon pack.
    Pack(PackArgs),
    /// Manage project work items.
    Work(WorkArgs),
    /// Inspect run history or one run folder.
    Log(LogArgs),
    /// Record a stage-gate approval or denial as a run artifact.
    Approve(ApproveArgs),
    /// Execute a canonical verifier suite and optionally retain its evidence.
    Verify(VerifyArgs),
    /// Manage portable cross-session project state.
    State(StateArgs),
    /// Internal harness-hook entrypoint.
    #[command(name = "__hook", hide = true)]
    Hook(HookArgs),
}

#[derive(Debug, Args)]
pub struct AdoptArgs {
    /// Pack ID (folder name under BASE_HOME/canon/packs/).
    pub pack: String,

    /// Replace an adopted pack with a strictly newer, immutable library version.
    #[arg(long)]
    pub upgrade: bool,
}

#[derive(Debug, Args)]
pub struct IngestArgs {
    /// Path to the source system's root (a Claude Code project or plugin).
    pub path: PathBuf,

    /// Record the inventory and mapping report under this existing run folder.
    #[arg(long, value_name = "RUN")]
    pub run: Option<String>,
}

#[derive(Debug, Args)]
pub struct PackArgs {
    #[command(subcommand)]
    pub command: PackCommand,
}

#[derive(Debug, Subcommand)]
pub enum PackCommand {
    /// Scaffold an empty library pack skeleton under BASE_HOME/canon/packs/.
    New {
        /// Pack ID (folder name under BASE_HOME/canon/packs/).
        id: String,
    },
    /// Validate a drafted pack's manifest, paths, and canonical frontmatter before adoption.
    Check {
        /// Path to the pack root (the directory containing pack.md).
        path: PathBuf,
    },
}

#[derive(Debug, Args)]
pub struct ApproveArgs {
    /// Run slug (folder name under .base/runs/).
    pub run: String,

    /// Stage-approval gate ID.
    pub gate: String,

    /// Record a denial instead of an approval.
    #[arg(long)]
    pub deny: bool,

    /// Who is deciding. Defaults to git user.name, then the OS username.
    #[arg(long, value_name = "WHO")]
    pub by: Option<String>,

    /// Context for the record, e.g. the standing directive being cited.
    #[arg(long, value_name = "TEXT")]
    pub note: Option<String>,
}

#[derive(Debug, Args)]
pub struct VerifyArgs {
    /// Verifier suite ID from canonical verifiers.
    pub suite: String,

    /// Record the report under this existing run folder.
    #[arg(long, value_name = "RUN")]
    pub run: Option<String>,
}

#[derive(Debug, Args)]
pub struct StateArgs {
    #[command(subcommand)]
    pub command: StateCommand,
}

#[derive(Debug, Subcommand)]
pub enum StateCommand {
    /// Show current work and the durable handoff.
    Show,
    /// Point current-work at an existing work item.
    Set { id: String },
    /// Clear the current-work pointer without deleting the work item.
    Clear,
    /// Emit concise context suitable for a session-start hook.
    Context,
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Initialize the user-wide canon at BASE_HOME (normally ~/.base).
    #[arg(long, conflicts_with = "project")]
    pub global: bool,

    /// Install or refresh only bundled global library packs, preserving personal canon.
    #[arg(long, requires = "global")]
    pub packs_only: bool,

    /// Initialize a project in the selected directory even outside git.
    #[arg(long, conflicts_with = "global")]
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
    #[command(name = "capabilities")]
    Capabilities {
        /// Require this executable's package version to satisfy a semantic-version range.
        #[arg(long, value_name = "RANGE")]
        require: Option<String>,

        /// Require a named hook-protocol feature.
        #[arg(long = "require-feature", value_name = "FEATURE")]
        require_features: Vec<String>,
    },
    #[command(name = "pre-tool")]
    PreTool {
        #[arg(long)]
        target: crate::config::Target,

        #[arg(long, default_value = "main")]
        default_branch: String,
    },
    #[command(name = "policy")]
    Policy {
        id: String,

        #[arg(long)]
        target: crate::config::Target,
    },
    /// Backward-compatible entrypoint for already-rendered Claude surfaces.
    #[command(name = "claude-pre-tool")]
    ClaudePreTool {
        #[arg(long, default_value = "main")]
        default_branch: String,
    },
}
