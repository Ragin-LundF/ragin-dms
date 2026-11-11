use std::sync::Arc;

use dms_domain::change::MissingFile;

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;
use crate::port::storage::{DocumentStorage, FolderOperation};
use crate::sync::Replicator;

/// Copies a catalogued file back into the folders it is missing from, taking it from a folder that still has it.
/// The default answer to a file deleted outside the tool.
pub struct RestoreDocument {
    store: Arc<CatalogStore>,
    storage: Arc<dyn DocumentStorage>,
    replicator: Arc<Replicator>,
    activity: Arc<ActivityRecorder>,
}

impl RestoreDocument {
    pub fn new(
        store: Arc<CatalogStore>,
        storage: Arc<dyn DocumentStorage>,
        replicator: Arc<Replicator>,
        activity: Arc<ActivityRecorder>,
    ) -> Self {
        Self { store, storage, replicator, activity }
    }

    pub fn run(&self, change: &MissingFile) -> OperationOutcome {
        let Some(library) = self.store.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        let Some(document) = library.catalog.documents().get(&change.document_id) else {
            return OperationOutcome::NotFound;
        };
        if document.path != change.path {
            return OperationOutcome::ScanOutdated;
        }
        let primary = library.folders.primary();
        if change.missing_in.contains(&primary.id) {
            let Some(source) = library.folders.folders().iter().find(|folder| change.present_in.contains(&folder.id))
            else {
                return OperationOutcome::NotFound;
            };
            if let Err(error) = self.storage.copy_file(source, primary, &change.path, false) {
                return OperationOutcome::StorageFailed(error);
            }
        }
        let targets: Vec<_> = library
            .folders
            .sync_folders()
            .into_iter()
            .filter(|folder| change.missing_in.contains(&folder.id))
            .collect();
        let pending = self.replicator.distribute(
            &library.folders,
            &targets,
            &FolderOperation::CopyFromPrimary { path: change.path.clone(), replace_existing: false },
        );
        self.activity.record(ActivityKind::DocumentRestored, change.path.value());
        OperationOutcome::Done { catalog: library.catalog, pending_folders: pending }
    }
}
