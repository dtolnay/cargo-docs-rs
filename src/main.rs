mod metadata;
mod parser;

use crate::metadata::Metadata;
use crate::parser::{Doc, Subcommand};
use anyhow::{bail, Context as _, Result};
use clap::Parser;
use std::collections::BTreeMap as Map;
use std::env;
use std::io::{self, Write as _};
use std::process::{self, Command, Stdio};

fn main() {
    if let Err(error) = do_main() {
        let _ = writeln!(io::stderr(), "Error: {:?}", error);
        process::exit(1);
    }
}

fn do_main() -> Result<()> {
    let Subcommand::Doc(args) = Subcommand::parse();

    let mut cargo_metadata = cargo_command();
    cargo_metadata.arg("metadata");
    cargo_metadata.arg("--format-version=1");
    propagate_common_args(&mut cargo_metadata, &args);
    cargo_metadata.stdin(Stdio::null());
    cargo_metadata.stdout(Stdio::piped());
    cargo_metadata.stderr(Stdio::inherit());
    let output = cargo_metadata.output()?;
    if !output.status.success() {
        process::exit(output.status.code().unwrap_or(1));
    }

    let metadata: Metadata = serde_json::from_slice(&output.stdout)
        .context("Failed to parse output of `cargo metadata`")?;

    let mut packages = Map::new();
    for pkg in metadata.packages {
        packages.insert(pkg.id.clone(), pkg);
    }

    let root = match metadata.resolve.root {
        Some(root) => root,
        None => {
            let mut options = String::new();
            for (i, member) in metadata.workspace_members.iter().enumerate() {
                options += if i == 0 { "" } else { " | " };
                options += &packages[&member].name;
            }
            bail!(
                "Pass `-p [{}]` to select a single workspace member",
                options,
            );
        }
    };

    println!("{:#?}", packages[&root]);
    Ok(())
}

fn cargo_command() -> Command {
    match env::var_os("CARGO") {
        Some(env) => Command::new(env),
        None => Command::new("cargo"),
    }
}

// Args that are meaningful to both `cargo metadata` and `cargo doc`.
fn propagate_common_args(cargo: &mut Command, args: &Doc) {
    if let Some(manifest_path) = &args.manifest_path {
        cargo.arg("--manifest-path");
        cargo.arg(manifest_path);
    }

    if args.frozen {
        cargo.arg("--frozen");
    }

    if args.locked {
        cargo.arg("--locked");
    }

    if args.offline {
        cargo.arg("--offline");
    }
}
