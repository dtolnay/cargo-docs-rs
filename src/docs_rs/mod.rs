// https://github.com/rust-lang/docs.rs/blob/2f67be0ed1f3c8d84d2a6c48b7d102598090d864/src/web/mod.rs

use std::{path::PathBuf, process::Command, sync::Arc};

use crate::metadata::DocumentationOptions;

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

pub mod context;
#[allow(clippy::module_inception)]
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
    {
        context.storage.store_one(
            "robots.txt".into(),
            include_str!("../../docs.rs/static/robots.txt").into(),
        )?;
        context.storage.store_one(
            "favicon.ico".into(),
            include_bytes!("../../docs.rs/static/favicon.ico").into(),
        )?;
        context.storage.store_one(
            "opensearch.xml".into(),
            include_str!("../../docs.rs/static/opensearch.xml").into(),
        )?;
        for (path, content) in generated_code::raw_static() {
            context.storage.store_one(
                "-/static".parse::<PathBuf>().unwrap().join(path),
                content.into(),
            )?;
        }
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
    }

    // "/sitemap.xml"
    // TODO generate sitemap.xml

    // "/-/sitemap/:letter/sitemap.xml"
    // TODO generate sitemap.xml

    // "/about/builds"
    // TODO generate builds.html

    // "/about/metrics/instance"
    // TODO generate instance.html

    // "/about/metrics/service"
    // TODO generate service.html

    // "/about/metrics"
    // TODO generate metrics.html

    // "/about"
    // TODO generate about.html

    // "about/:subpage"
    // TODO generate subpage.html

    // "/"
    // TODO generate index.html

    // "/releases"
    // TODO generate releases.html

    // "/releases/recent/:page"
    // TODO generate page.html

    // "/releases/stars"
    // TODO generate stars.html

    // "/releases/stars/:page"
    // TODO generate page.html

    // "/releases/recent-failures"
    // TODO generate recent-failures.html

    // "/releases/recent-failures/:page"
    // TODO generate page.html

    // "/releases/failures"
    // TODO generate failures.html

    // "/releases/failures/:page"
    // TODO generate page.html

    // "/crate/:name"
    {
        let url: PathBuf = format!(
            "crate/{}/{}.html",
            context.package.name, context.package.version
        )
        .parse()?;
        let page = crate_details_handler(
            CrateDetailHandlerParams {
                name: context.package.name.clone(),
                version: Some(context.package.version.to_string()),
            },
            context.clone(),
        )
        .await?;
        context.storage.store_one(url, page.clone().into())?;
        let url: PathBuf = format!("crate/{}/latest.html", context.package.name).parse()?;
        context.storage.store_one(url, page.into())?;
    }

    // "/crate/:name/:version"
    // TODO generate version.html

    // "/releases/feed"
    // TODO generate feed.xml

    // "/releases/:owner"
    // TODO generate owner.html

    // "/releases/:owner/:page"
    // TODO generate page.html

    // "/releases/activity"
    // TODO generate activity.html

    // "/releases/search"
    // TODO generate search.html

    // "/releases/queue"
    // TODO generate queue.html

    // "/crate/:name/:version/builds"
    // TODO generate builds.html

    // "/crate/:name/:version/builds.json"
    // TODO generate builds.json

    // "/crate/:name/:version/status.json"
    // TODO generate status.json

    // "/crate/:name/:version/builds/:id"
    // TODO generate build.html

    // "/crate/:name/:version/features"
    // TODO generate features.html

    // "/crate/:name/:version/source/"
    // TODO generate source.html

    // "/crate/:name/:version/source/*path"
    // TODO generate source.html

    // "/crate/:name/:version/menus/platforms/crate/"
    // TODO generate crate.html

    // "/crate/:name/:version/menus/platforms/crate/features"
    // TODO generate features.html

    // "/crate/:name/:version/menus/platforms/crate/builds"
    // TODO generate builds.html

    // "/crate/:name/:version/menus/platforms/crate/builds/*path"
    // TODO generate builds.html

    // "/crate/:name/:version/menus/platforms/crate/source/"
    // TODO generate source.html

    // "/crate/:name/:version/menus/platforms/crate/source/*path"
    // TODO generate source.html

    // "/crate/:name/:version/menus/platforms/:target"
    // TODO generate target.html

    // "/crate/:name/:version/menus/platforms/:target/*path"
    // TODO generate target.html

    // "/crate/:name/:version/menus/platforms/"
    // TODO generate platforms.html

    // "/crate/:name/:version/menus/platforms/:target/"
    // TODO generate target.html

    // "/crate/:name/:version/menus/releases/:target"
    // TODO generate target.html

    // "/crate/:name/:version/menus/releases/:target/*path"
    // TODO generate target.html

    // "/crate/:name/:version/menus/releases"
    // TODO generate releases.html

    // "/crate/:name/:version/menus/releases/:target/"
    // TODO generate target.html

    // "/-/rustdoc.static/*path"
    // TODO generate rustdoc.static.html

    // "/-/storage-change-detection.html"
    // TODO generate storage-change-detection.html

    // "/crate/:name/:version/download"
    // TODO generate download.html

    // "/crate/:name/:version/target-redirect/*path"
    // TODO generate target-redirect.html

    // "/:name/badge.svg"
    // TODO generate badge.svg

    // "/:name"
    // TODO generate name.html

    // "/:name/"
    // TODO generate index.html

    // "/:name/:version"
    // TODO generate version.html

    // "/:name/:version/"
    // TODO generate index.html

    // "/:name/:version/all.html"
    // TODO generate all.html

    // "/:name/:version/help.html"
    // TODO generate help.html

    // "/:name/:version/settings.html"
    // TODO generate settings.html

    // "/:name/:version/scrape-examples-help.html"
    // TODO generate scrape-examples-help.html

    // "/:name/:version/:target"
    // TODO generate target.html

    // "/:name/:version/:target/"
    // TODO generate index.html

    // "/:name/:version/:target/*path"
    // TODO generate target.html

    Ok(())
}
