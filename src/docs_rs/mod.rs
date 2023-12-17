// https://github.com/rust-lang/docs.rs/blob/2f67be0ed1f3c8d84d2a6c48b7d102598090d864/src/web/mod.rs

use std::{path::PathBuf, process::Command, sync::Arc};

use crate::{
    docs_rs::docs_rs::utils::rustc_version::parse_rustc_version, metadata::DocumentationOptions,
};

use self::{
    context::Context,
    docs_rs::{
        config::Config,
        index::Index,
        registry_api::RegistryApi,
        storage::Storage,
        web::{
            crate_details::{crate_details_handler, CrateDetailHandlerParams},
            page::{GlobalAlert, TemplateData},
        },
    },
};
use anyhow::Result;
use cargo_metadata::MetadataCommand;
use log::info;
use semver::Version;
use url::Url;

pub mod context;
pub mod docs_rs;
pub mod generated_code;

// https://github.com/rust-lang/docs.rs/blob/2f67be0ed1f3c8d84d2a6c48b7d102598090d864/src/web/mod.rs

pub(crate) static GLOBAL_ALERT: Option<GlobalAlert> = None;
pub const BUILD_VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " ",
    include_str!(concat!(env!("OUT_DIR"), "/git_version"))
);

#[allow(unused)]
pub(crate) fn generate_static_servers(
    dir: PathBuf,
    pkg_name: String,
    doc: DocumentationOptions,
    max_targets: usize,
) -> Result<()> {
    let metadata = MetadataCommand::new().exec()?;
    let mut pkg = metadata
        .packages
        .iter()
        .find(|p| p.name == pkg_name)
        .unwrap()
        .clone();
    info!("pkg: {:?}", pkg);

    let output = Command::new("rustc")
        .arg("--version")
        .output()
        .expect("Failed to execute rustc");

    // Check if the command was successful
    if !output.status.success() {
        println!("Error: {:?}", output.stderr);
    }
    let version_string = String::from_utf8(output.stdout)
        .expect("Invalid UTF-8")
        .trim()
        .to_string();

    let template_data = TemplateData::new()?;

    let config = Config::from_env()?;

    let registry_api = RegistryApi::new(
        config.registry_api_host.clone(),
        config.crates_io_api_call_retries,
    )?;

    let context = Context {
        config: Arc::new(config),
        storage: Arc::new(Storage::new(dir)),
        index: Arc::new(Index::new(PathBuf::from("index"))?),
        registry_api: Arc::new(registry_api),
        // repository_stats_updater: Arc::new(RepositoryStatsUpdater::new()?),
        templates: Arc::new(template_data),
        package: Arc::new(pkg),
        doc: Arc::new(doc),
        max_targets,
        rustc_version: version_string,
        metadata: Arc::new(metadata),
    };

    tokio::runtime::Runtime::new()?.block_on(routes(context))?;

    Ok(())
}

// src/web/routes.rs
async fn routes(context: Context) -> Result<()> {
    // hint for naming axum routes:
    // when routes overlap, the route parameters at the same position
    // have to use the same name:
    //
    // These routes work together:
    // - `/:name/:version/settings.html`
    // - `/:name/:version/:target`
    // and axum can prioritize the more specific route.
    //
    // This panics because of conflicting routes:
    // - `/:name/:version/settings.html`
    // - `/:crate/:version/:target`
    //
    // Well known resources, robots.txt and favicon.ico support redirection, the sitemap.xml
    // must live at the site root:
    //   https://developers.google.com/search/reference/robots_txt#handling-http-result-codes
    //   https://support.google.com/webmasters/answer/183668?hl=en

    // static files
    context.storage.store_one(
        "-/static/rustdoc-2021-12-05.css".into(),
        include_str!(concat!(env!("OUT_DIR"), "/rustdoc-2021-12-05.css")).into(),
    )?;
    context.storage.store_one(
        "-/static/rustdoc.css".into(),
        include_str!(concat!(env!("OUT_DIR"), "/rustdoc.css")).into(),
    )?;
    context.storage.store_one(
        "-/static/style.css".into(),
        include_str!(concat!(env!("OUT_DIR"), "/style.css")).into(),
    )?;
    context.storage.store_one(
        "-/static/vendored.css".into(),
        include_str!(concat!(env!("OUT_DIR"), "/vendored.css")).into(),
    )?;

    // "/crate/:name"
    {
        let url: PathBuf = format!("crate/{}/index.html", context.package.name).parse()?;
        let page = crate_details_handler(
            CrateDetailHandlerParams {
                name: context.package.name.clone(),
                version: Some(context.package.version.to_string()),
            },
            context.clone(),
        )
        .await?;
        context.storage.store_one(url, page)?;
    }

    Ok(())
}
