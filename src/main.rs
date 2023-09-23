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

    let metadata = &packages[&root].metadata;

    let mut cargo_rustdoc = cargo_command();
    cargo_rustdoc.arg("rustdoc");
    cargo_rustdoc.arg("-Zunstable-options");
    cargo_rustdoc.arg("-Zrustdoc-map");
    cargo_rustdoc.arg("-Zhost-config");
    cargo_rustdoc.arg("-Ztarget-applies-to-host");
    propagate_common_args(&mut cargo_rustdoc, &args);
    cargo_rustdoc.env("DOCS_RS", "1");

    cargo_rustdoc.arg("--lib");
    if let Some(package) = &args.package {
        cargo_rustdoc.arg("--package");
        cargo_rustdoc.arg(package);
    }

    if !metadata.features.is_empty() {
        cargo_rustdoc.arg("--features");
        cargo_rustdoc.arg(metadata.features.join(","));
    }

    if metadata.all_features {
        cargo_rustdoc.arg("--all-features");
    }

    if metadata.no_default_features {
        cargo_rustdoc.arg("--no-default-features");
    }

    if !args.target.is_empty() {
        for target in args.target {
            cargo_rustdoc.arg("--target");
            cargo_rustdoc.arg(target);
        }
    } else if args.open {
        // When using `--open`, only a single target is supported.
        if let Some(default_target) = &metadata.default_target {
            cargo_rustdoc.arg("--target");
            cargo_rustdoc.arg(default_target);
        } else if let Some(targets) = &metadata.targets {
            if let Some(default_target) = targets.first() {
                cargo_rustdoc.arg("--target");
                cargo_rustdoc.arg(default_target);
            }
        }
    } else if let Some(targets) = &metadata.targets {
        for target in targets {
            cargo_rustdoc.arg("--target");
            cargo_rustdoc.arg(target);
        }
    } else if let Some(default_target) = &metadata.default_target {
        cargo_rustdoc.arg("--target");
        cargo_rustdoc.arg(default_target);
    }

    cargo_rustdoc.arg("--config");
    cargo_rustdoc.arg(format!(
        "build.rustflags={}",
        toml::Value::try_from(&metadata.rustc_args).unwrap(),
    ));

    cargo_rustdoc.arg("--config");
    cargo_rustdoc.arg(format!(
        "host.rustflags={}",
        toml::Value::try_from(&metadata.rustc_args).unwrap(),
    ));

    let mut rustdocflags = metadata.rustdoc_args.clone();
    rustdocflags.insert(0, "-Zunstable-options".to_owned());
    rustdocflags.push("--extern-html-root-takes-precedence".to_owned());

    cargo_rustdoc.arg("--config");
    cargo_rustdoc.arg(format!(
        "build.rustdocflags={}",
        toml::Value::try_from(rustdocflags).unwrap(),
    ));

    cargo_rustdoc.arg("--config");
    cargo_rustdoc.arg("doc.extern-map.registries.crates-io=\"https://docs.rs\"");

    cargo_rustdoc.args(&metadata.cargo_args);

    if let Some(jobs) = args.jobs {
        cargo_rustdoc.arg("--jobs");
        cargo_rustdoc.arg(jobs.to_string());
    }

    if let Some(target_dir) = &args.target_dir {
        cargo_rustdoc.arg("--target-dir");
        cargo_rustdoc.arg(target_dir);
    }

    if args.open {
        cargo_rustdoc.arg("--open");
    }

    let status = cargo_rustdoc.status()?;
    if !status.success() {
        process::exit(status.code().unwrap_or(1));
    }

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
