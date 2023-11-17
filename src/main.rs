#![allow(
    clippy::struct_excessive_bools,
    clippy::too_many_lines,
    clippy::uninlined_format_args
)]

mod metadata;
mod parser;

use crate::metadata::{DocumentationOptions, Metadata};
use crate::parser::{Doc, Subcommand};
use anyhow::{bail, Context as _, Result};
use clap::{CommandFactory as _, Parser as _};
use std::collections::BTreeMap as Map;
use std::env;
use std::io::{self, Write as _};
use std::process::{self, Command, Stdio};

cargo_subcommand_metadata::description!("Immitate the documentation build that docs.rs would do");

fn main() {
    if let Err(error) = do_main() {
        let _ = writeln!(io::stderr(), "Error: {:?}", error);
        process::exit(1);
    }
}

fn do_main() -> Result<()> {
    let Subcommand::Doc(args) = Subcommand::parse();

    if args.version {
        let mut stdout = io::stdout();
        let _ = stdout.write_all(Subcommand::command().render_version().as_bytes());
        return Ok(());
    }

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

    let default_documentation_options = DocumentationOptions::default();
    let mut proc_macro = false;
    let metadata = if let Some(package) = &args.package {
        let mut package_metadata = &default_documentation_options;
        for workspace_member in &metadata.workspace_members {
            if packages[workspace_member].name == *package {
                let package = &packages[workspace_member];
                proc_macro = package.is_proc_macro();
                package_metadata = &package.metadata;
                break;
            }
        }
        package_metadata
    } else if let Some(root) = metadata.resolve.root {
        let package = &packages[&root];
        proc_macro = package.is_proc_macro();
        &package.metadata
    } else {
        let mut options = String::new();
        for (i, member) in metadata.workspace_members.iter().enumerate() {
            options += if i == 0 { "" } else { " | " };
            options += &packages[&member].name;
        }
        bail!(
            "Pass `-p [{}]` to select a single workspace member",
            options,
        );
    };

    let mut doc_targets = Vec::new();
    if !args.target.is_empty() {
        for target in &args.target {
            doc_targets.push(target);
        }
    } else if proc_macro {
        // Ignore selected target because proc macro can only be built for host.
    } else if args.open {
        // When using `--open`, only a single target is supported.
        if let Some(default_target) = &metadata.default_target {
            doc_targets.push(default_target);
        } else if let Some(targets) = &metadata.targets {
            if let Some(default_target) = targets.first() {
                doc_targets.push(default_target);
            }
        }
    } else if let Some(targets) = &metadata.targets {
        for target in targets {
            doc_targets.push(target);
        }
    } else if let Some(default_target) = &metadata.default_target {
        doc_targets.push(default_target);
    }

    for target in &doc_targets {
        let mut child = Command::new("rustc")
            .arg("-")
            .arg("--target")
            .arg(target)
            .arg("-Zunpretty=expanded")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .context("failed to spawn rustc")?;
        let _ = child.stdin.unwrap().write_all(b"#![no_std]\n");
        child.stdin = None; // close
        let status = child
            .wait()
            .context("failed to wait for rustc subcommand")?;
        if !status.success() {
            process::exit(status.code().unwrap_or(1));
        }
    }

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

    for target in &doc_targets {
        cargo_rustdoc.arg("--target");
        cargo_rustdoc.arg(target);
    }

    let mut rustflags = metadata.rustc_args.clone();
    if let Some(encoded_rustflags) = env::var_os("CARGO_ENCODED_RUSTFLAGS") {
        if let Some(encoded_rustflags) = encoded_rustflags.to_str() {
            rustflags.splice(0..0, encoded_rustflags.split('\x1f').map(str::to_owned));
        }
    } else if let Some(env_rustflags) = env::var_os("RUSTFLAGS") {
        if let Some(env_rustflags) = env_rustflags.to_str() {
            rustflags.splice(0..0, env_rustflags.split_whitespace().map(str::to_owned));
        }
    }

    cargo_rustdoc.arg("--config");
    cargo_rustdoc.arg(format!(
        "build.rustflags={}",
        toml::Value::try_from(&rustflags).unwrap(),
    ));

    cargo_rustdoc.arg("--config");
    cargo_rustdoc.arg(format!(
        "host.rustflags={}",
        toml::Value::try_from(&rustflags).unwrap(),
    ));

    let mut rustdocflags = metadata.rustdoc_args.clone();
    rustdocflags.insert(0, "-Zunstable-options".to_owned());
    if let Some(encoded_rustdocflags) = env::var_os("CARGO_ENCODED_RUSTDOCFLAGS") {
        if let Some(encoded_rustdocflags) = encoded_rustdocflags.to_str() {
            rustdocflags.splice(1..1, encoded_rustdocflags.split('\x1f').map(str::to_owned));
        }
    } else if let Some(env_rustdocflags) = env::var_os("RUSTDOCFLAGS") {
        if let Some(env_rustdocflags) = env_rustdocflags.to_str() {
            rustdocflags.splice(1..1, env_rustdocflags.split_whitespace().map(str::to_owned));
        }
    }
    rustdocflags.push("--extern-html-root-takes-precedence".to_owned());

    cargo_rustdoc.arg("--config");
    cargo_rustdoc.arg(format!(
        "build.rustdocflags={}",
        toml::Value::try_from(&rustdocflags).unwrap(),
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

    if args.verbose {
        cargo_rustdoc.arg("--verbose");
    }

    cargo_rustdoc.env_remove("RUSTFLAGS");
    cargo_rustdoc.env_remove("RUSTDOCFLAGS");
    cargo_rustdoc.env_remove("CARGO_ENCODED_RUSTFLAGS");
    cargo_rustdoc.env_remove("CARGO_ENCODED_RUSTDOCFLAGS");

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
