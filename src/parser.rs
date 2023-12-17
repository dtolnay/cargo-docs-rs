use clap::{Parser, ValueEnum};
use std::path::PathBuf;

const PACKAGE_SELECTION: &str = "Package Selection";
const COMPILATION_OPTIONS: &str = "Compilation Options";
const MANIFEST_OPTIONS: &str = "Manifest Options";
#[cfg(feature = "accessory")]
const DOCS_RS_OPTIONS: &str = "docs.rs Options";

#[derive(Parser)]
#[command(bin_name = "cargo", version, author, disable_help_subcommand = true)]
pub enum Subcommand {
    /// Imitate the documentation build that docs.rs would do
    #[command(name = "docs-rs", version, author, disable_version_flag = true)]
    Doc(Doc),
}

#[derive(Parser, Debug)]
pub struct Doc {
    /// Opens the docs in a browser after the operation
    #[arg(long)]
    pub open: bool,

    /// Print command lines as they are executed
    #[arg(short, long)]
    pub verbose: bool,

    /// Print version
    #[arg(long)]
    pub version: bool,

    /// Package to document
    #[arg(short, long, value_name = "SPEC", help_heading = PACKAGE_SELECTION)]
    pub package: Option<String>,

    /// Number of parallel jobs, defaults to # of CPUs
    #[arg(short, long, value_name = "N", help_heading = COMPILATION_OPTIONS)]
    pub jobs: Option<u64>,

    /// Build for the target triple
    #[arg(long, value_name = "TARGET", help_heading = COMPILATION_OPTIONS)]
    pub target: Vec<String>,

    /// Directory for all generated artifacts
    #[arg(long, value_name = "DIRECTORY", help_heading = COMPILATION_OPTIONS)]
    pub target_dir: Option<PathBuf>,

    /// Path to Cargo.toml
    #[arg(long, value_name = "PATH", help_heading = MANIFEST_OPTIONS)]
    pub manifest_path: Option<PathBuf>,

    /// Require Cargo.lock and cache are up to date
    #[arg(long, help_heading = MANIFEST_OPTIONS)]
    pub frozen: bool,

    /// Require Cargo.lock is up to date
    #[arg(long, help_heading = MANIFEST_OPTIONS)]
    pub locked: bool,

    /// Run without accessing the network
    #[arg(long, help_heading = MANIFEST_OPTIONS)]
    pub offline: bool,

    #[cfg(feature = "accessory")]
    /// Use Accessory
    #[arg(long, help_heading = DOCS_RS_OPTIONS)]
    pub accessory: bool,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum Coloring {
    Auto,
    Always,
    Never,
}

#[test]
fn test_cli() {
    <Subcommand as clap::CommandFactory>::command().debug_assert();
}
