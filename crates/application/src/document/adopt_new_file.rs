use std::sync::Arc;

use dms_domain::change::NewFile;
use dms_domain::document::{Document, DocumentId};

use crate::common::{ActivityRecorder, DocumentDates, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;
use crate::port::storage::{DocumentStorage, FolderOperation};
use crate::port::system::{Clock, IdGenerator};
use crate::sync::Replicator;

/// Catalogs a file the scan found but the catalog does not know, and copies it into every folder that lacks it:
/// into the primary from a folder that has it, then from the primary into the sync folders. The file keeps its
/// name and place.
pub struct AdoptNewFile {
    store: Arc<CatalogStore>,
    storage: Arc<dyn DocumentStorage>,
    dates: Arc<DocumentDates>,
    replicator: Arc<Replicator>,
    activity: Arc<ActivityRecorder>,
    clock: Arc<dyn Clock>,
    ids: Arc<dyn IdGenerator>,
}

impl AdoptNewFile {
    pub fn new(
        store: Arc<CatalogStore>,
        storage: Arc<dyn DocumentStorage>,
        dates: Arc<DocumentDates>,
        replicator: Arc<Replicator>,
        activity: Arc<ActivityRecorder>,
        clock: Arc<dyn Clock>,
        ids: Arc<dyn IdGenerator>,
    ) -> Self {
        Self { store, storage, dates, replicator, activity, clock, ids }
    }

    pub fn run(&self, change: &NewFile) -> OperationOutcome {
        let Some(library) = self.store.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        let Some(content_hash) = change.content_hash.clone() else {
            return OperationOutcome::ScanOutdated;
        };
        let primary = library.folders.primary();
        // Where the file was found first: there its creation date is still the original one.
        let Some(source) = library.folders.folders().iter().find(|folder| change.present_in.contains(&folder.id))
        else {
            return OperationOutcome::ScanOutdated;
        };
        let metadata = self.dates.metadata_for(source, &change.path);
        if !change.present_in.contains(&primary.id)
            && let Err(error) = self.storage.copy_file(source, primary, &change.path, false)
        {
            return OperationOutcome::StorageFailed(error);
        }
        let targets: Vec<_> = library
            .folders
            .sync_folders()
            .into_iter()
            .filter(|folder| !change.present_in.contains(&folder.id))
            .collect();
        let pending = self.replicator.distribute(
            &library.folders,
            &targets,
            &FolderOperation::CopyFromPrimary { path: change.path.clone(), replace_existing: false },
        );
        let document = Document {
            metadata,
            ..Document::new(
                DocumentId::new(self.ids.new_id()).expect("invariant: generated ids are not blank"),
                change.path.clone(),
                content_hash,
                change.size,
                self.clock.now(),
            )
            .expect("invariant: a scanned file is never the root")
        };
        let outcome =
            self.store.commit(|catalog| catalog.add_document(document.clone())).with_pending_folders(&pending);
        if matches!(outcome, OperationOutcome::Done { .. }) {
            self.activity.record(ActivityKind::DocumentAdded, change.path.value());
        }
        outcome
    }
}
