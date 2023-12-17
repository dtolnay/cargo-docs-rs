// https://github.com/rust-lang/docs.rs/blob/7fdd5d839cb68d703c2732d784aa12692d58ab54/src/web/crate_details.rs#L37

use std::sync::Arc;

use crate::docs_rs::docs_rs::storage::{PathNotFoundError, Storage};
use crate::docs_rs::docs_rs::utils::rustc_version::get_correct_docsrs_style_file;
use crate::metadata::DocumentationOptions;
use anyhow::{Context as AnyhowContext, Result};
use cargo_metadata::Package;
use chrono::{DateTime, Utc};
use log::warn;
use serde::Deserialize;
use serde::{ser::Serializer, Serialize};
use serde_json::{json, Value};
use url::Url;

use super::csp::Csp;
use super::page::web_page::TemplateRender;
use super::rustdoc::RustdocHtmlParams;
use super::{markdown, MetaData};

// TODO: Add target name and versions

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CrateDetails {
    name: String,
    version: String,
    description: Option<String>,
    owners: Vec<(String, String)>,
    dependencies: Option<Value>,
    #[serde(serialize_with = "optional_markdown")]
    readme: Option<String>,
    #[serde(serialize_with = "optional_markdown")]
    rustdoc: Option<String>, // this is description_long in database
    release_time: DateTime<Utc>,
    build_status: bool,
    pub latest_build_id: Option<i32>,
    last_successful_build: Option<String>,
    pub rustdoc_status: bool,
    // pub archive_storage: bool,
    repository_url: Option<String>,
    homepage_url: Option<String>,
    keywords: Option<Value>,
    have_examples: bool, // need to check this manually
    pub target_name: String,
    releases: Vec<Release>,
    repository_metadata: Option<RepositoryMetadata>,
    pub(crate) metadata: MetaData,
    is_library: bool,
    license: Option<String>,
    pub(crate) documentation_url: Option<String>,
    total_items: Option<i32>,
    documented_items: Option<i32>,
    total_items_needing_examples: Option<i32>,
    items_with_examples: Option<i32>,
    /// Database id for this crate
    pub(crate) crate_id: i32,
    /// Database id for this release
    pub(crate) release_id: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct RepositoryMetadata {
    stars: i32,
    forks: i32,
    issues: i32,
    name: Option<String>,
    icon: &'static str,
}

fn optional_markdown<S>(markdown: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    markdown
        .as_ref()
        .map(|markdown| markdown::render(markdown))
        .serialize(serializer)
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct Release {
    pub id: i32,
    pub version: semver::Version,
    pub build_status: bool,
    pub yanked: bool,
    pub is_library: bool,
    pub rustdoc_status: bool,
    pub target_name: String,
}

impl CrateDetails {
    pub fn new(
        pkg: Arc<Package>,
        options: Arc<DocumentationOptions>,
        ctx: crate::docs_rs::context::Context,
    ) -> Result<Option<CrateDetails>, anyhow::Error> {
        // get releases, sorted by semver
        // let releases: Vec<Release> = releases_for_crate(conn, krate.crate_id).await?;
        let releases = Vec::new();

        // let repositry = metadata.root_package().unwrap().repository.unwrap();
        // let repository_metadata: RepositoryMetadata = RepositoryMetadata {
        //     issues: krate.repo_issues.unwrap(),
        //     stars: krate.repo_stars.unwrap(),
        //     forks: krate.repo_forks.unwrap(),
        //     name: krate.repo_name,
        //     icon: up.map_or("code-branch", |u| u.get_icon_name(&host)),
        // };
        let repository_metadata = None;

        let metadata = MetaData {
            name: pkg.name.clone(),
            version: pkg.version.to_string(),
            version_or_latest: "latest".to_string(),
            description: pkg.description.clone(),
            rustdoc_status: false,
            target_name: Some(pkg.name.clone()),
            default_target: options
                .default_target
                .clone()
                .unwrap_or(String::from("x86_64-unknown-linux-gnu")),
            doc_targets: options.targets.clone().unwrap_or(Vec::new()),
            yanked: false,
            rustdoc_css_file: get_correct_docsrs_style_file(&ctx.rustc_version)?,
        };

        let mut crate_details = CrateDetails {
            name: pkg.name.clone(),
            version: pkg.version.to_string(),
            description: pkg.description.clone(),
            owners: Vec::new(),
            dependencies: Some(
                pkg.dependencies
                    .clone()
                    .into_iter()
                    .map(|x| {
                        Value::Array(vec![
                            Value::String(x.name.clone()),
                            Value::String(
                                ctx.metadata
                                    .packages
                                    .iter()
                                    .find(|p| p.name == x.name)
                                    .unwrap()
                                    .version
                                    .to_string(),
                            ),
                            Value::String(x.kind.to_string()),
                        ])
                    })
                    .collect::<Value>(),
            ),
            readme: pkg.readme.clone().map(|readme| String::from(readme)),
            rustdoc: pkg.description.clone(),
            release_time: Utc::now(),
            build_status: false,
            latest_build_id: None,
            last_successful_build: None,
            rustdoc_status: false,
            // archive_storage: false,
            repository_url: pkg.repository.clone(),
            homepage_url: pkg.homepage.clone(),
            keywords: Some(
                pkg.keywords
                    .clone()
                    .into_iter()
                    .map(|keywords| serde_json::Value::from(keywords))
                    .collect(),
            ),
            have_examples: false,          // TODO: check this manually
            target_name: pkg.name.clone(), // TODO: what  is this
            releases,
            repository_metadata,
            metadata,
            is_library: true, // TODO: check this manually
            license: pkg.license.clone(),
            documentation_url: pkg.documentation.clone(),
            documented_items: None,
            total_items: None,                  // TODO: check this manually
            total_items_needing_examples: None, // TODO: check this manually
            items_with_examples: None,          // TODO: check this manually
            crate_id: 0,                        // TODO: what is this
            release_id: 0,                      // TODO: nothing
        };

        // get owners
        // crate_details.owners = sqlx::query!(
        //     "SELECT login, avatar
        //      FROM owners
        //      INNER JOIN owner_rels ON owner_rels.oid = owners.id
        //      WHERE cid = $1",
        //     pkg.crate_id,
        // )
        // .fetch(&mut *conn)
        // .map_ok(|row| (row.login, row.avatar))
        // .try_collect()
        // .await?;

        if !crate_details.build_status {
            crate_details.last_successful_build = crate_details
                .releases
                .iter()
                .filter(|release| release.build_status && !release.yanked)
                .map(|release| release.version.to_string())
                .next();
        }

        Ok(Some(crate_details))
    }

    async fn fetch_readme(&self, storage: Arc<Storage>) -> anyhow::Result<Option<String>> {
        let manifest = match storage.fetch_source_file(
            &self.name,
            &self.version,
            self.latest_build_id.unwrap_or(0),
            "Cargo.toml",
        ) {
            Ok(manifest) => manifest,
            Err(err) if err.is::<PathNotFoundError>() => {
                return Ok(None);
            }
            Err(err) => {
                return Err(err);
            }
        };
        let manifest = String::from_utf8(manifest.content)
            .context("parsing Cargo.toml")?
            .parse::<toml::Value>()
            .context("parsing Cargo.toml")?;
        let paths = match manifest.get("package").and_then(|p| p.get("readme")) {
            Some(toml::Value::Boolean(true)) => vec!["README.md"],
            Some(toml::Value::Boolean(false)) => vec![],
            Some(toml::Value::String(path)) => vec![path.as_ref()],
            _ => vec!["README.md", "README.txt", "README"],
        };
        for path in &paths {
            match storage.fetch_source_file(
                &self.name,
                &self.version,
                self.latest_build_id.unwrap_or(0),
                path,
            ) {
                Ok(readme) => {
                    let readme = String::from_utf8(readme.content)
                        .with_context(|| format!("parsing {path} content"))?;
                    return Ok(Some(readme));
                }
                Err(err) if err.is::<PathNotFoundError>() => {
                    continue;
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
        Ok(None)
    }

    /// Returns the latest non-yanked, non-prerelease release of this crate (or latest
    /// yanked/prereleased if that is all that exist).
    pub fn latest_release(&self) -> &Release {
        self.releases
            .iter()
            .find(|release| release.version.pre.is_empty() && !release.yanked)
            .unwrap_or(&self.releases[0])
    }
}

/// Return all releases for a crate, sorted in descending order by semver
// pub(crate) async fn releases_for_crate(
//     conn: &mut sqlx::PgConnection,
//     crate_id: i32,
// ) -> Result<Vec<Release>, anyhow::Error> {
//     let mut releases: Vec<Release> = sqlx::query!(
//         "SELECT
//             id,
//             version,
//             build_status,
//             yanked,
//             is_library,
//             rustdoc_status,
//             target_name
//          FROM releases
//          WHERE
//              releases.crate_id = $1",
//         crate_id,
//     )
//     .fetch(&mut *conn)
//     .try_filter_map(|row| async move {
//         Ok(
//             match semver::Version::parse(&row.version).with_context(|| {
//                 format!(
//                     "invalid semver in database for crate {crate_id}: {}",
//                     row.version
//                 )
//             }) {
//                 Ok(semversion) => Some(Release {
//                     id: row.id,
//                     version: semversion,
//                     build_status: row.build_status,
//                     yanked: row.yanked,
//                     is_library: row.is_library,
//                     rustdoc_status: row.rustdoc_status,
//                     target_name: row.target_name,
//                 }),
//                 Err(err) => {
//                     report_error(&err);
//                     None
//                 }
//             },
//         )
//     })
//     .try_collect()
//     .await?;

//     releases.sort_by(|a, b| b.version.cmp(&a.version));
//     Ok(releases)
// }

#[derive(Debug, Clone, PartialEq, Serialize)]
struct CrateDetailsPage {
    details: CrateDetails,
}

// impl_axum_webpage! {
//     CrateDetailsPage = "crate/details.html",
//     cpu_intensive_rendering = true,
// }

#[derive(Deserialize, Clone, Debug)]
pub(crate) struct CrateDetailHandlerParams {
    pub(crate) name: String,
    pub(crate) version: Option<String>,
}

// #[tracing::instrument(skip(conn, storage))]
pub(crate) async fn crate_details_handler(
    params: CrateDetailHandlerParams,
    ctx: crate::docs_rs::context::Context,
) -> Result<String> {
    // this handler must always called with a crate name
    // if params.version.is_none() {
    //     return Ok(super::axum_cached_redirect(
    //         encode_url_path(&format!("/crate/{}/latest", params.name)),
    //         CachePolicy::ForeverInCdn,
    //     )?
    //     .into_response());
    // }

    // let found_version = match_version(&mut conn, &params.name, params.version.as_deref())
    //     .await
    //     .and_then(|m| m.exact_name_only())?;

    // let (version, version_or_latest, is_latest_url) = match found_version {
    //     MatchSemver::Exact((version, _)) => (version.clone(), version, false),
    //     MatchSemver::Latest((version, _)) => (version, "latest".to_string(), true),
    //     MatchSemver::Semver((version, _)) => {
    //         return Ok(super::axum_cached_redirect(
    //             &format!("/crate/{}/{}", &params.name, version),
    //             CachePolicy::ForeverInCdn,
    //         )?
    //         .into_response());
    //     }
    // };

    let mut details =
        CrateDetails::new(ctx.package.clone(), ctx.doc.clone(), ctx.clone())?.unwrap();
    // .ok_or(AxumNope::VersionNotFound)?;

    match details.fetch_readme(ctx.storage).await {
        Ok(readme) => details.readme = readme.or(details.readme),
        Err(e) => warn!("error fetching readme: {:?}", &e),
    }

    let res = CrateDetailsPage { details };
    // res.extensions_mut()
    //     .insert::<CachePolicy>(if is_latest_url {
    //         CachePolicy::ForeverInCdn
    //     } else {
    //         CachePolicy::ForeverInCdnAndStaleInBrowser
    //     });

    let render = TemplateRender {
        template: String::from("crate/details.html"),
        context: {
            let mut context = tera::Context::from_serialize(&res)?;
            context.insert("max_targets", &ctx.max_targets);
            context
        },
    }
    .render_response(ctx.templates, Csp::new().nonce().into())?;

    Ok(render)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct ReleaseList {
    releases: Vec<Release>,
    crate_name: String,
    inner_path: String,
    target: String,
}

// impl_axum_webpage! {
//     ReleaseList = "rustdoc/releases.html",
//     cache_policy = |_| CachePolicy::ForeverInCdn,
//     cpu_intensive_rendering = true,
// }

// #[tracing::instrument]
// pub(crate) async fn get_all_releases(
//     Path(params): Path<RustdocHtmlParams>,
//     mut conn: DbConnection,
// ) -> AxumResult<AxumResponse> {
//     let req_path: String = params.path.clone().unwrap_or_default();
//     let req_path: Vec<&str> = req_path.split('/').collect();

//     let release_found = match_version(&mut conn, &params.name, Some(&params.version)).await?;
//     trace!(?release_found, "found release");

//     let (version, _) = match release_found.version {
//         MatchSemver::Exact((version, _)) => (version.clone(), version),
//         MatchSemver::Latest((version, _)) => (version, "latest".to_string()),
//         MatchSemver::Semver(_) => return Err(AxumNope::VersionNotFound),
//     };

//     let row = sqlx::query!(
//         "SELECT
//             crates.id AS crate_id,
//             releases.doc_targets
//         FROM crates
//         INNER JOIN releases on crates.id = releases.crate_id
//         WHERE crates.name = $1 and releases.version = $2;",
//         params.name,
//         &version,
//     )
//     .fetch_optional(&mut *conn)
//     .await?
//     .ok_or(AxumNope::CrateNotFound)?;

//     // get releases, sorted by semver
//     let releases: Vec<Release> = releases_for_crate(&mut conn, row.crate_id).await?;

//     let doc_targets = MetaData::parse_doc_targets(row.doc_targets);

//     let inner;
//     let (target, inner_path) = {
//         let mut inner_path = req_path.clone();

//         let target = if inner_path.len() > 1
//             && doc_targets
//                 .iter()
//                 .any(|s| Some(s) == params.target.as_ref())
//         {
//             inner_path.remove(0);
//             params.target.as_ref().unwrap()
//         } else {
//             ""
//         };

//         inner = inner_path.join("/");
//         (target, inner.trim_end_matches('/'))
//     };
//     let inner_path = if inner_path.is_empty() {
//         format!("{}/index.html", params.name)
//     } else {
//         format!("{}/{inner_path}", params.name)
//     };

//     let target = if target.is_empty() {
//         String::new()
//     } else {
//         format!("{target}/")
//     };

//     let res = ReleaseList {
//         releases,
//         target: target.to_string(),
//         inner_path,
//         crate_name: params.name,
//     };
//     Ok(res.into_response())
// }

#[derive(Debug, Clone, PartialEq, Serialize)]
struct ShortMetadata {
    name: String,
    version_or_latest: String,
    doc_targets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct PlatformList {
    metadata: ShortMetadata,
    inner_path: String,
    use_direct_platform_links: bool,
    current_target: String,
}

// impl_axum_webpage! {
//     PlatformList = "rustdoc/platforms.html",
//     cache_policy = |_| CachePolicy::ForeverInCdn,
//     cpu_intensive_rendering = true,
// }

// #[tracing::instrument]
pub(crate) async fn get_all_platforms_inner(
    params: RustdocHtmlParams,
    ctx: crate::docs_rs::context::Context,
    is_crate_root: bool,
) -> Result<String> {
    let req_path: String = params.path.unwrap_or_default();
    let req_path: Vec<&str> = req_path.split('/').collect();

    // let release_found = match_version(&mut conn, &params.name, Some(&params.version)).await?;
    // trace!(?release_found, "found release");

    // Convenience function to allow for easy redirection
    // #[instrument]
    // fn redirect(
    //     name: &str,
    //     vers: &str,
    //     path: &[&str],
    // ) -> AxumResult<AxumResponse> {
    //     trace!("redirect");
    //     // Format and parse the redirect url
    //     Ok(axum_cached_redirect(
    //         encode_url_path(&format!("/platforms/{}/{}/{}", name, vers, path.join("/"))),
    //         cache_policy,
    //     )?
    //     .into_response())
    // }

    // let (version, version_or_latest) = match release_found.version {
    //     MatchSemver::Exact((version, _)) => {
    //         // Redirect when the requested crate name isn't correct
    //         if let Some(name) = release_found.corrected_name {
    //             return redirect(&name, &version, &req_path, CachePolicy::NoCaching);
    //         }

    //         (version.clone(), version)
    //     }

    //     MatchSemver::Latest((version, _)) => {
    //         // Redirect when the requested crate name isn't correct
    //         if let Some(name) = release_found.corrected_name {
    //             return redirect(&name, "latest", &req_path, CachePolicy::NoCaching);
    //         }

    //         (version, "latest".to_string())
    //     }

    //     // Redirect when the requested version isn't correct
    //     MatchSemver::Semver((v, _)) => {
    //         // to prevent cloudfront caching the wrong artifacts on URLs with loose semver
    //         // versions, redirect the browser to the returned version instead of loading it
    //         // immediately
    //         return redirect(&params.name, &v, &req_path, CachePolicy::ForeverInCdn);
    //     }
    // };

    let version = ctx.package.version.to_string();

    // let releases = releases_for_crate(&mut conn, krate.id).await?;

    let doc_targets = ctx.doc.targets.clone().unwrap_or(Vec::new());

    // let latest_release = releases
    //     .iter()
    //     .find(|release| release.version.pre.is_empty() && !release.yanked)
    //     .unwrap_or(&releases[0]);

    // The path within this crate version's rustdoc output
    let inner;
    let (target, inner_path) = {
        let mut inner_path = req_path.clone();

        let target = if inner_path.len() > 1
            && doc_targets
                .iter()
                .any(|s| Some(s) == params.target.as_ref())
        {
            inner_path.remove(0);
            params.target.as_ref().unwrap()
        } else {
            ""
        };

        inner = inner_path.join("/");
        (target, inner.trim_end_matches('/'))
    };
    let inner_path = if inner_path.is_empty() {
        format!("{}/index.html", ctx.package.name)
    } else {
        format!("{}/{inner_path}", ctx.package.name)
    };

    // let current_target = if latest_release.build_status {
    //     if target.is_empty() {
    //         krate.default_target
    //     } else {
    //         target.to_owned()
    //     }
    // } else {
    //     String::new()
    // };

    let res = PlatformList {
        metadata: ShortMetadata {
            name: ctx.package.name.clone(),
            version_or_latest: ctx.package.version.to_string(),
            doc_targets,
        },
        inner_path,
        use_direct_platform_links: is_crate_root,
        current_target: ctx.doc.default_target.clone().unwrap_or(String::new()),
    };

    let render = TemplateRender {
        template: String::from("rustdoc/platforms.html"),
        context: {
            let mut context = tera::Context::from_serialize(&res)?;
            context.insert("max_targets", &ctx.max_targets);
            context
        },
    }
    .render_response(ctx.templates, Csp::new().nonce().into())?;

    Ok(render)
}

pub(crate) async fn get_all_platforms_root(
    mut params: RustdocHtmlParams,
    ctx: crate::docs_rs::context::Context,
) -> Result<String> {
    params.path = None;
    get_all_platforms_inner(params, ctx, true).await
}

pub(crate) async fn get_all_platforms(
    params: RustdocHtmlParams,
    ctx: crate::docs_rs::context::Context,
) -> Result<String> {
    get_all_platforms_inner(params, ctx, false).await
}
