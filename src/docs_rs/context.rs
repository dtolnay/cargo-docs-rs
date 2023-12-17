use std::sync::Arc;

use cargo_metadata::{Metadata, Package};

use crate::metadata::DocumentationOptions;

use super::docs_rs::{
    config::Config, index::Index, registry_api::RegistryApi, storage::Storage,
    web::page::TemplateData,
};

#[derive(Clone)]
pub struct Context {
    pub(crate) config: Arc<Config>,
    pub(crate) storage: Arc<Storage>,
    pub(crate) index: Arc<Index>,
    pub(crate) registry_api: Arc<RegistryApi>,
    // pub(crate) repository_stats_updater: Arc<RepositoryStatsUpdater>,
    pub(crate) templates: Arc<TemplateData>,
    pub(crate) package: Arc<Package>,
    pub(crate) doc: Arc<DocumentationOptions>,
    pub(crate) max_targets: usize,
    pub(crate) rustc_version: String,
    pub(crate) metadata: Arc<Metadata>,
}
