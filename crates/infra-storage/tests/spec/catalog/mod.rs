mod catalog_details_format;
mod catalog_fixtures;
mod catalog_organize_format;
mod catalog_round_trip;
mod file_catalog_repository;

use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use dms_application::port::storage::{CatalogLoad, CatalogRepository};
use dms_domain::catalog::Catalog;
use dms_domain::document::{Document, DocumentId};
use dms_domain::library::LibraryFolder;
use dms_infra_storage::FileCatalogRepository;
use tempfile::TempDir;

use crate::support::{RecordingTrash, folder, hash_of, path};

/// A library folder in a temporary directory with a repository and a recording trash.
pub struct Library {
    pub temp: TempDir,
    pub root: PathBuf,
    pub folder: LibraryFolder,
    pub trash: Arc<RecordingTrash>,
    pub repository: FileCatalogRepository,
}

impl Library {
    pub fn new() -> Self {
        let temp = tempfile::tempdir().expect("temp dir");
        let root = temp.path().join("library");
        std::fs::create_dir_all(&root).expect("library folder");
        let trash = Arc::new(RecordingTrash::new(&temp.path().join("trash")));
        let repository = FileCatalogRepository::new(trash.clone());
        Self { folder: folder(&root), root, temp, trash, repository }
    }

    pub fn file(&self) -> PathBuf {
        self.root.join("dms-catalog.yaml")
    }

    /// Saves `catalog` as a change based on its own revision.
    pub fn save(&self, catalog: &Catalog) {
        self.repository.save(&self.folder, catalog, catalog.revision()).expect("saved");
    }

    pub fn found(&self) -> Catalog {
        match self.repository.load(&self.folder) {
            CatalogLoad::Found { catalog, .. } => catalog,
            other => panic!("expected a catalog, got {other:?}"),
        }
    }

    /// Writes `text` as the catalog file and loads it.
    pub fn load_text(&self, text: &str) -> Catalog {
        std::fs::write(self.file(), text).expect("catalog file");
        self.found()
    }

    pub fn text(&self) -> String {
        std::fs::read_to_string(self.file()).expect("catalog file")
    }
}

pub fn instant(text: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(text).expect("instant").with_timezone(&Utc)
}

pub fn document(id: &str, raw_path: &str, content: &str) -> Document {
    Document::new(
        DocumentId::new(id).expect("id"),
        path(raw_path),
        hash_of(content),
        content.len() as u64,
        instant("2024-01-01T10:00:00Z"),
    )
    .expect("document")
}
