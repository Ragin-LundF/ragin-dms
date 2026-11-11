use std::sync::Arc;

use dms_domain::DomainError;
use dms_domain::catalog::{Catalog, CatalogUpdate};
use dms_domain::document::{Document, DocumentId, DocumentMetadata};
use dms_domain::library::LibraryFolder;
use indexmap::IndexSet;

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::state::{ActivityKind, TaskStore};
use crate::port::storage::DocumentStorage;

/// Keeps one of several catalogued copies of a file. The others go to the system trash in every folder and leave
/// the catalog; what the user assigned to them (category, tags, title, date, correspondent, the AI classification)
/// moves to the kept document where it has nothing yet.
///
/// Only true copies go: the catalog must know them with the same size and SHA-256, and right before anything
/// moves, each folder compares the two files byte by byte. If a copy differs in the primary, nothing changes at
/// all. A sync folder where the comparison fails, or that is offline, keeps its file, and the next scan reports it
/// again. Only with the user's confirmation.
pub struct MergeDuplicates {
    store: Arc<CatalogStore>,
    storage: Arc<dyn DocumentStorage>,
    tasks: Arc<dyn TaskStore>,
    activity: Arc<ActivityRecorder>,
}

impl MergeDuplicates {
    pub fn new(
        store: Arc<CatalogStore>,
        storage: Arc<dyn DocumentStorage>,
        tasks: Arc<dyn TaskStore>,
        activity: Arc<ActivityRecorder>,
    ) -> Self {
        Self { store, storage, tasks, activity }
    }

    pub fn run(&self, keep: &DocumentId, confirmed: bool) -> OperationOutcome {
        if !confirmed {
            return OperationOutcome::NotConfirmed;
        }
        let Some(library) = self.store.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        let Some(kept) = library.catalog.documents().get(keep).cloned() else {
            return OperationOutcome::NotFound;
        };
        let copies: Vec<Document> = library
            .catalog
            .find_by_content_hash(&kept.content_hash)
            .into_iter()
            .filter(|copy| copy.id != kept.id && copy.size == kept.size)
            .cloned()
            .collect();
        let primary = library.folders.primary();
        if copies.is_empty() || !copies.iter().all(|copy| self.identical(primary, &kept, copy)) {
            return OperationOutcome::ScanOutdated;
        }
        for copy in &copies {
            if let Err(error) = self.storage.move_to_trash(primary, &copy.path) {
                return OperationOutcome::StorageFailed(error);
            }
            for folder in library.folders.sync_folders() {
                if self.identical(folder, &kept, copy) {
                    let _ = self.storage.move_to_trash(folder, &copy.path);
                }
            }
        }
        let outcome = self.store.commit(|catalog| merged(catalog, &kept, &copies));
        if matches!(outcome, OperationOutcome::Done { .. }) {
            for copy in &copies {
                // Best effort: the copy is gone, a stale suggestion only waits for nothing.
                let _ = self.tasks.remove_suggestion(&copy.id);
                self.activity.record(ActivityKind::DocumentTrashed, copy.path.value());
            }
        }
        outcome
    }

    fn identical(&self, folder: &LibraryFolder, kept: &Document, copy: &Document) -> bool {
        self.storage.same_content(folder, &kept.path, &copy.path) == Ok(true)
    }
}

fn merged(catalog: &Catalog, kept: &Document, copies: &[Document]) -> Result<CatalogUpdate, DomainError> {
    let metadata = DocumentMetadata {
        title: kept.metadata.title.clone().or_else(|| copies.iter().find_map(|copy| copy.metadata.title.clone())),
        document_date: kept
            .metadata
            .document_date
            .or_else(|| copies.iter().find_map(|copy| copy.metadata.document_date)),
        correspondent: kept
            .metadata
            .correspondent
            .clone()
            .or_else(|| copies.iter().find_map(|copy| copy.metadata.correspondent.clone())),
    };
    let category_id = kept.category_id.clone().or_else(|| copies.iter().find_map(|copy| copy.category_id.clone()));
    let classification =
        kept.classification.clone().or_else(|| copies.iter().find_map(|copy| copy.classification.clone()));
    let tags: IndexSet<_> = kept.tags.iter().chain(copies.iter().flat_map(|copy| copy.tags.iter())).cloned().collect();
    let mut update = copies.iter().fold(CatalogUpdate::Applied(catalog.clone()), |update, copy| {
        update.and_then(|current| current.remove_document(&copy.id))
    });
    if let CatalogUpdate::Applied(current) = update {
        update = current.update_metadata(&kept.id, metadata)?;
    }
    update = update
        .and_then(|current| current.assign_category(&kept.id, category_id))
        .and_then(|current| current.set_tags(&kept.id, &tags));
    match (update, classification) {
        (CatalogUpdate::Applied(current), Some(record)) => current.record_classification(&kept.id, record),
        (update, _) => Ok(update),
    }
}
