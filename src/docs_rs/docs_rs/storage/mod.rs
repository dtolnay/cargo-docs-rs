// https://github.com/rust-lang/docs.rs/blob/2f67be0ed1f3c8d84d2a6c48b7d102598090d864/src/storage/mod.rs

use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use log::trace;
use std::{
    fs::{self, File},
    ops::RangeInclusive,
    path::{Path, PathBuf},
};

type FileRange = RangeInclusive<u64>;

#[derive(Debug, thiserror::Error)]
#[error("path not found")]
pub(crate) struct PathNotFoundError;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Blob {
    pub(crate) path: String,
    pub(crate) mime: String,
    pub(crate) date_updated: DateTime<Utc>,
    pub(crate) content: Vec<u8>,
}

impl Blob {
    pub(crate) fn from_file(path: PathBuf, file: &mut File) -> Result<Self> {
        let metadata = file.metadata()?;
        let date_updated = metadata.modified().map_err(|_| PathNotFoundError)?.into();
        let mime = mime_guess::from_path(&path)
            .first_raw()
            .unwrap_or("application/octet-stream")
            .to_string();
        let content = std::fs::read(path.clone())?;
        Ok(Self {
            path: path.to_string_lossy().to_string(),
            mime,
            date_updated,
            content,
        })
    }
}

pub struct Storage {
    pub(crate) path: PathBuf,
}

#[allow(dead_code)]
impl Storage {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub(crate) fn exists(&self, path: &str) -> Result<bool> {
        let path = self.path.join(path);
        Ok(path.exists())
    }

    /// Fetch a rustdoc file from our blob storage.
    /// * `name` - the crate name
    /// * `version` - the crate version
    /// * `latest_build_id` - the id of the most recent build. used purely to invalidate the local archive
    ///   index cache, when `archive_storage` is `true.` Without it we wouldn't know that we have
    ///   to invalidate the locally cached file after a rebuild.
    /// * `path` - the wanted path inside the documentation.
    /// * `archive_storage` - if `true`, we will assume we have a remove ZIP archive and an index
    ///    where we can fetch the requested path from inside the ZIP file.
    /// * `fetch_time` - used to collect metrics when using the storage inside web server handlers.
    pub(crate) fn fetch_rustdoc_file(
        &self,
        name: &str,
        version: &str,
        latest_build_id: i32,
        path: &str,
    ) -> Result<Blob> {
        trace!("fetch rustdoc file");
        // Add rustdoc prefix, name and version to the path for accessing the file stored in the database
        let remote_path = format!("rustdoc/{name}/{version}/{path}");
        self.get(&remote_path)
    }

    pub(crate) fn fetch_source_file(
        &self,
        name: &str,
        version: &str,
        latest_build_id: i32,
        path: &str,
    ) -> Result<Blob> {
        let remote_path = format!("sources/{name}/{version}/{path}");
        self.get(&remote_path)
    }

    pub(crate) fn rustdoc_file_exists(
        &self,
        name: &str,
        version: &str,
        latest_build_id: i32,
        path: &str,
    ) -> Result<bool> {
        // Add rustdoc prefix, name and version to the path for accessing the file stored in the database
        let remote_path = format!("rustdoc/{name}/{version}/{path}");
        self.exists(&remote_path)
    }

    pub(crate) fn get(&self, path: &str) -> Result<Blob> {
        let path = self.path.join(path);
        let mut file = File::open(&path)?;
        let mut blob = Blob::from_file(path, &mut file)?;
        Ok(blob)
    }

    // pub(super) fn get_range(&self, path: &str, range: FileRange) -> Result<Blob> {
    //     let mut blob = match &self.backend {
    //         StorageBackend::Database(db) => db.get(path, max_size, Some(range)).await,
    //         StorageBackend::S3(s3) => s3.get(path, max_size, Some(range)).await,
    //     }?;
    //     // `compression` represents the compression of the file-stream inside the archive.
    //     // We don't compress the whole archive, so the encoding of the archive's blob is irrelevant
    //     // here.
    //     if let Some(alg) = compression {
    //         blob.content = decompress(blob.content.as_slice(), alg, max_size)?;
    //         blob.compression = None;
    //     }
    //     Ok(blob)
    // }

    pub(crate) fn store_one(&self, path: PathBuf, page: String) -> Result<()> {
        let docs_path = Path::new("./docs");
        if !docs_path.exists() {
            fs::create_dir(docs_path).expect("Failed to create docs directory");
        }
        let path = docs_path.to_path_buf().join(path);

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        if !path.exists() {
            File::create(&path)?;
        }
        std::fs::write(path, page)?;

        Ok(())
    }
}
