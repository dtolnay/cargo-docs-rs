use anyhow::{anyhow, bail, Context, Result};
use log::trace;
use std::{env::VarError, error::Error, path::PathBuf, str::FromStr, time::Duration};
use url::Url;

#[derive(Debug)]
pub struct Config {
    pub prefix: PathBuf,
    pub registry_index_path: PathBuf,
    pub registry_url: Option<String>,
    pub registry_api_host: Url,

    // Github authentication
    pub(crate) github_accesstoken: Option<String>,
    pub(crate) github_updater_min_rate_limit: u32,

    // Gitlab authentication
    pub(crate) gitlab_accesstoken: Option<String>,

    // amount of retries for external API calls, mostly crates.io
    pub crates_io_api_call_retries: u32,

    // Max size of the files served by the docs.rs frontend
    pub(crate) max_file_size: usize,
    pub(crate) max_file_size_html: usize,
    // Time between 'git gc --auto' calls in seconds
    pub(crate) registry_gc_interval: u64,

    // random crate search generates a number of random IDs to
    // efficiently find a random crate with > 100 GH stars.
    // The amount depends on the ratio of crates with >100 stars
    // to the count of all crates.
    // At the time of creating this setting, it is set to
    // `500` for a ratio of 7249 over 54k crates.
    // For unit-tests the number has to be higher.
    pub(crate) random_crate_search_view_size: u32,

    // where do we want to store the locally cached index files
    // for the remote archives?
    pub(crate) local_archive_cache_path: PathBuf,

    // Content Security Policy
    pub(crate) csp_report_only: bool,

    // Cache-Control header, for versioned URLs.
    // If both are absent, don't generate the header. If only one is present,
    // generate just that directive. Values are in seconds.
    pub(crate) cache_control_stale_while_revalidate: Option<u32>,

    // Activate full page caching.
    // When disabled, we still cache static assets.
    // This only affects pages that depend on invalidations to work.
    pub(crate) cache_invalidatable_responses: bool,

    // Build params
    pub(crate) build_attempts: u16,
    pub(crate) delay_between_build_attempts: Duration,
    pub(crate) rustwide_workspace: PathBuf,
    pub(crate) temp_dir: PathBuf,
    pub(crate) inside_docker: bool,
    pub(crate) include_default_targets: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let old_vars = [
            ("CRATESFYI_PREFIX", "DOCSRS_PREFIX"),
            ("CRATESFYI_DATABASE_URL", "DOCSRS_DATABASE_URL"),
            ("CRATESFYI_GITHUB_ACCESSTOKEN", "DOCSRS_GITHUB_ACCESSTOKEN"),
            ("CRATESFYI_RUSTWIDE_WORKSPACE", "DOCSRS_RUSTWIDE_WORKSPACE"),
            ("DOCS_RS_DOCKER", "DOCSRS_DOCKER"),
            ("DOCS_RS_LOCAL_DOCKER_IMAGE", "DOCSRS_DOCKER_IMAGE"),
            ("DOCS_RS_BULID_CPU_LIMIT", "DOCSRS_BULID_CPU_LIMIT"),
        ];
        for (old_var, new_var) in old_vars {
            if std::env::var(old_var).is_ok() {
                bail!(
                    "env variable {} is no longer accepted; use {} instead",
                    old_var,
                    new_var
                );
            }
        }

        let prefix: PathBuf = env("DOCSRS_PREFIX", "ignored/cratesfyi-prefix".parse()?)?;

        let temp_dir = prefix.join("tmp");

        Ok(Self {
            build_attempts: env("DOCSRS_BUILD_ATTEMPTS", 5)?,
            delay_between_build_attempts: Duration::from_secs(env::<u64>(
                "DOCSRS_DELAY_BETWEEN_BUILD_ATTEMPTS",
                60,
            )?),

            crates_io_api_call_retries: env("DOCSRS_CRATESIO_API_CALL_RETRIES", 3)?,

            registry_index_path: env("REGISTRY_INDEX_PATH", prefix.join("crates.io-index"))?,
            registry_url: maybe_env("REGISTRY_URL")?,
            registry_api_host: env(
                "DOCSRS_REGISTRY_API_HOST",
                "https://crates.io".parse().unwrap(),
            )?,
            prefix: prefix.clone(),

            github_accesstoken: maybe_env("DOCSRS_GITHUB_ACCESSTOKEN")?,
            github_updater_min_rate_limit: env("DOCSRS_GITHUB_UPDATER_MIN_RATE_LIMIT", 2500)?,

            gitlab_accesstoken: maybe_env("DOCSRS_GITLAB_ACCESSTOKEN")?,

            max_file_size: env("DOCSRS_MAX_FILE_SIZE", 50 * 1024 * 1024)?,
            max_file_size_html: env("DOCSRS_MAX_FILE_SIZE_HTML", 50 * 1024 * 1024)?,
            // LOL HTML only uses as much memory as the size of the start tag!
            // https://github.com/rust-lang/docs.rs/pull/930#issuecomment-667729380
            registry_gc_interval: env("DOCSRS_REGISTRY_GC_INTERVAL", 60 * 60)?,

            random_crate_search_view_size: env("DOCSRS_RANDOM_CRATE_SEARCH_VIEW_SIZE", 500)?,

            csp_report_only: env("DOCSRS_CSP_REPORT_ONLY", false)?,

            cache_control_stale_while_revalidate: maybe_env(
                "CACHE_CONTROL_STALE_WHILE_REVALIDATE",
            )?,

            cache_invalidatable_responses: env("DOCSRS_CACHE_INVALIDATEABLE_RESPONSES", true)?,

            local_archive_cache_path: env(
                "DOCSRS_ARCHIVE_INDEX_CACHE_PATH",
                prefix.join("archive_cache"),
            )?,

            temp_dir,

            rustwide_workspace: env("DOCSRS_RUSTWIDE_WORKSPACE", PathBuf::from(".workspace"))?,
            inside_docker: env("DOCSRS_DOCKER", false)?,
            include_default_targets: env("DOCSRS_INCLUDE_DEFAULT_TARGETS", true)?,
        })
    }
}

fn env<T>(var: &str, default: T) -> Result<T>
where
    T: FromStr,
    T::Err: Error + Send + Sync + 'static,
{
    Ok(maybe_env(var)?.unwrap_or(default))
}

fn require_env<T>(var: &str) -> Result<T>
where
    T: FromStr,
    <T as FromStr>::Err: Error + Send + Sync + 'static,
{
    maybe_env(var)?.with_context(|| anyhow!("configuration variable {} is missing", var))
}

fn maybe_env<T>(var: &str) -> Result<Option<T>>
where
    T: FromStr,
    T::Err: Error + Send + Sync + 'static,
{
    match std::env::var(var) {
        Ok(content) => Ok(content
            .parse::<T>()
            .map(Some)
            .with_context(|| format!("failed to parse configuration variable {var}"))?),
        Err(VarError::NotPresent) => {
            trace!("optional configuration variable {} is not set", var);
            Ok(None)
        }
        Err(VarError::NotUnicode(_)) => Err(anyhow!("configuration variable {} is not UTF-8", var)),
    }
}
