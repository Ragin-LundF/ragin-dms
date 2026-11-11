use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::Arc;

use dms_domain::catalog::{Catalog, CatalogRejection, CatalogUpdate};
use dms_domain::document::{Document, DocumentId, DocumentMetadata};
use dms_domain::library::RelativePath;
use indexmap::IndexMap;

use crate::common::{ActivityRecorder, DocumentDates, OperationOutcome};
use crate::library::{CatalogStore, ScannedFile};
use crate::port::state::ActivityKind;
use crate::port::system::{Clock, IdGenerator};

/// Catalogs files of the primary folder while a scan reads them, many in one catalog write. The files keep their
/// name and place; nothing on disk changes except the catalog. Their date is guessed by [`DocumentDates`]. A file
/// whose path or content the catalog already knows is left to the change detection, because it may be a moved
/// document.
pub struct AddScannedFiles {
    store: Arc<CatalogStore>,
    dates: Arc<DocumentDates>,
    activity: Arc<ActivityRecorder>,
    clock: Arc<dyn Clock>,
    ids: Arc<dyn IdGenerator>,
}

impl AddScannedFiles {
    pub fn new(
        store: Arc<CatalogStore>,
        dates: Arc<DocumentDates>,
        activity: Arc<ActivityRecorder>,
        clock: Arc<dyn Clock>,
        ids: Arc<dyn IdGenerator>,
    ) -> Self {
        Self { store, dates, activity, clock, ids }
    }

    /// Returns how many of `files` were added.
    pub fn run(&self, files: &[ScannedFile]) -> usize {
        let Some(library) = self.store.current() else {
            return 0;
        };
        if files.is_empty() {
            return 0;
        }
        let primary = library.folders.primary();
        let metadata: IndexMap<RelativePath, DocumentMetadata> =
            files.iter().map(|file| (file.path.clone(), self.dates.metadata_for(primary, &file.path))).collect();
        let added: RefCell<Vec<Document>> = RefCell::new(Vec::new());
        let outcome = self.store.commit(|catalog| {
            let mut current = catalog.clone();
            let mut accepted = Vec::new();
            for document in self.new_documents(catalog, files, &metadata) {
                if let CatalogUpdate::Applied(next) = current.add_document(document.clone())? {
                    current = next;
                    accepted.push(document);
                }
            }
            let empty = accepted.is_empty();
            *added.borrow_mut() = accepted;
            Ok(if empty {
                CatalogUpdate::Rejected(CatalogRejection::PathAlreadyUsed)
            } else {
                CatalogUpdate::Applied(current)
            })
        });
        if !matches!(outcome, OperationOutcome::Done { .. }) {
            return 0;
        }
        let added = added.into_inner();
        let subject = match added.as_slice() {
            [single] => single.path.value().to_owned(),
            _ => format!("{} files found by the scan", added.len()),
        };
        self.activity.record(ActivityKind::DocumentAdded, &subject);
        added.len()
    }

    fn new_documents(
        &self,
        catalog: &Catalog,
        files: &[ScannedFile],
        metadata: &IndexMap<RelativePath, DocumentMetadata>,
    ) -> Vec<Document> {
        let mut known_paths: HashSet<&RelativePath> = catalog.documents().values().map(|d| &d.path).collect();
        let mut known_hashes: HashSet<_> = catalog.documents().values().map(|d| &d.content_hash).collect();
        let now = self.clock.now();
        files
            .iter()
            // Both sets learn every file, like Kotlin's `add(path) && add(hash)` (the hash only when the path was new).
            .filter(|file| known_paths.insert(&file.path) && known_hashes.insert(&file.content_hash))
            .map(|file| Document {
                metadata: metadata.get(&file.path).cloned().unwrap_or_default(),
                ..Document::new(
                    DocumentId::new(self.ids.new_id()).expect("invariant: generated ids are not blank"),
                    file.path.clone(),
                    file.content_hash.clone(),
                    file.size,
                    now,
                )
                .expect("invariant: a scanned file is never the root")
            })
            .collect()
    }
}
