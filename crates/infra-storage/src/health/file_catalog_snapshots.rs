use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDateTime, Utc};
use dms_application::port::health::{CatalogSnapshot, CatalogSnapshotId, CatalogSnapshots};
use dms_application::port::storage::{StorageFailure, StorageResult};
use dms_domain::catalog::{Catalog, CatalogRevision, LibraryId};

use crate::catalog::mapper::catalog_mapper;
use crate::catalog::read_catalog;
use crate::fs::{OrIoError, atomic_files, failure};
use crate::yaml::emit;

const STAMP: &str = "%Y%m%dT%H%M%SZ";
const EXTENSION: &str = "yaml";

/// `snapshots/<library-id>/<UTC timestamp>-r<revision>.yaml` in the app data folder: each a catalog file as the
/// library folders hold it. These are the app's own copies, so removing one deletes it.
pub struct FileCatalogSnapshots {
    root: PathBuf,
}

impl FileCatalogSnapshots {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The folder of `library_id`; an id that is no plain name never becomes a path.
    fn folder_of(&self, library_id: &LibraryId) -> StorageResult<PathBuf> {
        let id = library_id.value();
        let plain = id.chars().all(|char| char.is_ascii_alphanumeric() || char == '-' || char == '_');
        if !plain {
            return Err(failure(StorageFailure::UnsupportedFormat));
        }
        Ok(self.root.join(id))
    }

    fn file_of(&self, library_id: &LibraryId, id: &CatalogSnapshotId) -> StorageResult<PathBuf> {
        Ok(self.folder_of(library_id)?.join(format!("{}.{EXTENSION}", id.value())))
    }
}

impl CatalogSnapshots for FileCatalogSnapshots {
    fn save(&self, catalog: &Catalog, taken_at: DateTime<Utc>) -> StorageResult<()> {
        let id = CatalogSnapshotId::new(format!("{}-r{}", taken_at.format(STAMP), catalog.revision()))
            .map_err(crate::yaml::YamlError::from)?;
        let file = self.file_of(catalog.library_id(), &id)?;
        let dto = catalog_mapper::to_dto(catalog, None);
        atomic_files::write(&file, emit(&dto.write()).as_bytes()).or_io()
    }

    fn list(&self, library_id: &LibraryId) -> StorageResult<Vec<CatalogSnapshot>> {
        let entries = match fs::read_dir(self.folder_of(library_id)?) {
            Ok(entries) => entries,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(crate::fs::io_error(&error)),
        };
        let mut snapshots: Vec<CatalogSnapshot> =
            entries.filter_map(Result::ok).filter_map(|entry| snapshot_of(&entry.path())).collect();
        snapshots
            .sort_by(|first, second| second.taken_at.cmp(&first.taken_at).then(second.revision.cmp(&first.revision)));
        Ok(snapshots)
    }

    fn load(&self, library_id: &LibraryId, id: &CatalogSnapshotId) -> StorageResult<Catalog> {
        let file = self.file_of(library_id, id)?;
        if !file.is_file() {
            return Err(failure(StorageFailure::NotFound));
        }
        read_catalog(&file)
    }

    fn remove(&self, library_id: &LibraryId, id: &CatalogSnapshotId) -> StorageResult<()> {
        match fs::remove_file(self.file_of(library_id, id)?) {
            Err(error) if error.kind() != ErrorKind::NotFound => Err(crate::fs::io_error(&error)),
            _ => Ok(()),
        }
    }
}

/// The snapshot a file name describes; other files (a leftover temporary file) are none.
fn snapshot_of(file: &Path) -> Option<CatalogSnapshot> {
    if file.extension()? != EXTENSION {
        return None;
    }
    let stem = file.file_stem()?.to_str()?;
    let (stamp, revision) = stem.split_once("-r")?;
    let taken_at = NaiveDateTime::parse_from_str(stamp, STAMP).ok()?.and_utc();
    let revision = CatalogRevision::new(revision.parse().ok()?);
    Some(CatalogSnapshot { id: CatalogSnapshotId::new(stem).ok()?, taken_at, revision })
}
