use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(
    name = "pontis",
    version = env!("CARGO_PKG_VERSION"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    long_about = None,
    disable_help_subcommand = true
)]
pub struct Cli {
    /// Left path for local comparison
    #[arg(value_name = "LEFT")]
    pub left: Option<PathBuf>,

    /// Right path for local comparison
    #[arg(value_name = "RIGHT")]
    pub right: Option<PathBuf>,

    /// Path to config TOML file
    #[arg(long, value_name = "FILE", global = true)]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Compare repository states (`pontis git --help` for details)
    Git(GitCommand),
}

#[derive(Args, Debug, Clone)]
pub struct GitCommand {
    /// Git repository path (default: current directory)
    #[arg(long, value_name = "PATH")]
    pub repo: Option<PathBuf>,

    /// Compare HEAD/revision against the index (staged changes only)
    #[arg(long)]
    pub staged: bool,

    /// Left revision for git compare (default: HEAD)
    #[arg(long, value_name = "REV")]
    pub rev: Option<String>,

    /// Compare two revisions directly
    #[arg(
        long,
        num_args = 2,
        value_names = ["LEFT_REV", "RIGHT_REV"],
        conflicts_with_all = ["staged", "rev"]
    )]
    pub diff: Option<Vec<String>>,

    /// Left directory prepared by git difftool for revision-pair compare
    #[arg(long, value_name = "PATH", requires_all = ["diff", "difftool_right_dir"])]
    pub difftool_left_dir: Option<PathBuf>,

    /// Right directory prepared by git difftool for revision-pair compare
    #[arg(long, value_name = "PATH", requires_all = ["diff", "difftool_left_dir"])]
    pub difftool_right_dir: Option<PathBuf>,
}
