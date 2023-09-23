mod parser;

use crate::parser::{Doc, Subcommand};
use anyhow::Result;
use clap::Parser;
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

    let _ = io::stdout().write_all(&output.stdout);
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
