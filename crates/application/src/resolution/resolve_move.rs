use std::sync::Arc;

use dms_domain::catalog::CatalogUpdate;
use dms_domain::change::MovedFile;
use dms_domain::library::LibraryFolder;
use indexmap::IndexSet;

use crate::common::{ActivityRecorder, LibrarySnapshot, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;
use crate::port::storage::{DocumentStorage, FolderOperation};
use crate::resolution::MoveDecision;
use crate::sync::Replicator;

/// Applies a move made outside the tool to the other folders and the catalog, or undoes it.
pub struct ResolveMove {
    store: Arc<CatalogStore>,
    storage: Arc<dyn DocumentStorage>,
    replicator: Arc<Replicator>,
    activity: Arc<ActivityRecorder>,
}

impl ResolveMove {
    pub fn new(
        store: Arc<CatalogStore>,
        storage: Arc<dyn DocumentStorage>,
        replicator: Arc<Replicator>,
        activity: Arc<ActivityRecorder>,
    ) -> Self {
        Self { store, storage, replicator, activity }
    }

    pub fn run(&self, change: &MovedFile, decision: MoveDecision) -> OperationOutcome {
        let Some(library) = self.store.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        let Some(document) = library.catalog.documents().get(&change.document_id) else {
            return OperationOutcome::NotFound;
        };
        if document.path != change.previous_path {
            return OperationOutcome::ScanOutdated;
        }
        let Some(folder) = library.folders.find(&change.folder_id).cloned() else {
            return OperationOutcome::NotFound;
        };
        match decision {
            MoveDecision::Apply => self.apply(&library, &folder, change),
            MoveDecision::Undo => self.undo(library, &folder, change),
        }
    }

    fn apply(&self, library: &LibrarySnapshot, folder: &LibraryFolder, change: &MovedFile) -> OperationOutcome {
        match library.catalog.move_document(&change.document_id, change.path.clone()) {
            Err(error) => return OperationOutcome::InvalidInput(error),
            Ok(CatalogUpdate::Rejected(reason)) => return OperationOutcome::Rejected(reason),
            Ok(CatalogUpdate::Applied(_)) => {}
        }
        let primary = library.folders.primary();
        if folder != primary
            && let Err(error) = self.storage.move_entry(primary, &change.previous_path, &change.path)
        {
            return OperationOutcome::StorageFailed(error);
        }
        let targets: Vec<_> = library.folders.sync_folders().into_iter().filter(|other| *other != folder).collect();
        let pending = self.replicator.distribute(
            &library.folders,
            &targets,
            &FolderOperation::Move { from: change.previous_path.clone(), to: change.path.clone() },
        );
        let outcome = self
            .store
            .commit(|catalog| catalog.move_document(&change.document_id, change.path.clone()))
            .with_pending_folders(&pending);
        if matches!(outcome, OperationOutcome::Done { .. }) {
            self.activity.record(ActivityKind::DocumentMoved, change.path.value());
        }
        outcome
    }

    fn undo(&self, library: LibrarySnapshot, folder: &LibraryFolder, change: &MovedFile) -> OperationOutcome {
        if let Err(error) = self.storage.move_entry(folder, &change.path, &change.previous_path) {
            return OperationOutcome::StorageFailed(error);
        }
        self.activity.record(ActivityKind::DocumentMoved, change.previous_path.value());
        OperationOutcome::Done { catalog: library.catalog, pending_folders: IndexSet::new() }
    }
}
