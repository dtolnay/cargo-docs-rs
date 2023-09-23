use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(bin_name = "cargo", version, author, disable_help_subcommand = true)]
pub enum Subcommand {
    /// Immitate the documentation build that docs.rs would do
    #[command(name = "docs-rs", version, author, disable_version_flag = true)]
    Doc(Doc),
}

#[derive(Parser, Debug)]
pub struct Doc {
    /// Opens the docs in a browser after the operation
    #[arg(long)]
    pub open: bool,

    /// Package to document
    #[arg(short, long, value_name = "SPEC")]
    pub package: Option<String>,

    /// Number of parallel jobs, defaults to # of CPUs
    #[arg(short, long, value_name = "N")]
    pub jobs: Option<u64>,

    /// Build for the target triple
    #[arg(long, value_name = "TARGET")]
    pub target: Vec<String>,

    /// Directory for all generated artifacts
    #[arg(long, value_name = "DIRECTORY")]
    pub target_dir: Option<PathBuf>,

    /// Path to Cargo.toml
    #[arg(long, value_name = "PATH")]
    pub manifest_path: Option<PathBuf>,

    /// Require Cargo.lock and cache are up to date
    #[arg(long)]
    pub frozen: bool,

    /// Require Cargo.lock is up to date
    #[arg(long)]
    pub locked: bool,

    /// Run without accessing the network
    #[arg(long)]
    pub offline: bool,

    /// Print version
    #[arg(long)]
    pub version: bool,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum Coloring {
    Auto,
    Always,
    Never,
}
