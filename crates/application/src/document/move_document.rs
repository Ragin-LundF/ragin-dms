use std::sync::Arc;

use dms_domain::catalog::CatalogUpdate;
use dms_domain::document::DocumentId;
use dms_domain::library::{FileNamePolicy, RelativePath};
use indexmap::IndexSet;

use crate::common::{ActivityRecorder, LibrarySnapshot, OperationOutcome, SiblingNames};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;
use crate::port::storage::{DocumentStorage, FolderOperation};
use crate::sync::{Replication, Replicator};

/// Moves or renames one document in every library folder. A rename that only changes the case of the name is
/// allowed; any other name clash is refused, never resolved by overwriting.
pub struct MoveDocument {
    store: Arc<CatalogStore>,
    storage: Arc<dyn DocumentStorage>,
    replicator: Arc<Replicator>,
    names: Arc<SiblingNames>,
    activity: Arc<ActivityRecorder>,
}

impl MoveDocument {
    pub fn new(
        store: Arc<CatalogStore>,
        storage: Arc<dyn DocumentStorage>,
        replicator: Arc<Replicator>,
        names: Arc<SiblingNames>,
        activity: Arc<ActivityRecorder>,
    ) -> Self {
        Self { store, storage, replicator, names, activity }
    }

    pub fn run(&self, document_id: &DocumentId, new_path: &RelativePath) -> OperationOutcome {
        let Some(library) = self.store.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        let Some(document) = library.catalog.documents().get(document_id) else {
            return OperationOutcome::NotFound;
        };
        if document.path == *new_path {
            return OperationOutcome::Done { catalog: library.catalog, pending_folders: IndexSet::new() };
        }
        if let Some(violation) = FileNamePolicy::check(new_path.file_name()) {
            return OperationOutcome::InvalidName(violation);
        }
        // Names first: a clash with a catalogued document is a taken name, the same as one that differs by case.
        match self.names.is_free(&library, new_path, Some(&document.path)) {
            Err(error) => return OperationOutcome::StorageFailed(error),
            Ok(false) => return OperationOutcome::NameTaken,
            Ok(true) => {}
        }
        match library.catalog.move_document(document_id, new_path.clone()) {
            Err(error) => return OperationOutcome::InvalidInput(error),
            Ok(CatalogUpdate::Rejected(reason)) => return OperationOutcome::Rejected(reason),
            Ok(CatalogUpdate::Applied(_)) => {}
        }
        let from = document.path.clone();
        self.move_file(&library, document_id, &from, new_path)
    }

    fn move_file(
        &self,
        library: &LibrarySnapshot,
        document_id: &DocumentId,
        from: &RelativePath,
        to: &RelativePath,
    ) -> OperationOutcome {
        let replication = self.replicator.replicate(
            &library.folders,
            |primary| self.storage.move_entry(primary, from, to),
            &FolderOperation::Move { from: from.clone(), to: to.clone() },
        );
        let pending = match replication {
            Replication::PrimaryFailed(error) => return OperationOutcome::StorageFailed(error),
            Replication::Done { pending_folders, .. } => pending_folders,
        };
        let outcome =
            self.store.commit(|catalog| catalog.move_document(document_id, to.clone())).with_pending_folders(&pending);
        if matches!(outcome, OperationOutcome::Done { .. }) {
            self.activity.record(ActivityKind::DocumentMoved, to.value());
        }
        outcome
    }
}
