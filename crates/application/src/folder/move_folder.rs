use std::sync::Arc;

use dms_domain::catalog::CatalogUpdate;
use dms_domain::library::{FileNamePolicy, RelativePath};

use crate::common::{ActivityRecorder, OperationOutcome, SiblingNames};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;
use crate::port::storage::{DocumentStorage, FolderOperation};
use crate::sync::{Replication, Replicator};

/// Moves or renames a folder with everything in it, in every library folder, and moves the paths of the documents
/// inside it in the catalog.
pub struct MoveFolder {
    store: Arc<CatalogStore>,
    storage: Arc<dyn DocumentStorage>,
    replicator: Arc<Replicator>,
    names: Arc<SiblingNames>,
    activity: Arc<ActivityRecorder>,
}

impl MoveFolder {
    pub fn new(
        store: Arc<CatalogStore>,
        storage: Arc<dyn DocumentStorage>,
        replicator: Arc<Replicator>,
        names: Arc<SiblingNames>,
        activity: Arc<ActivityRecorder>,
    ) -> Self {
        Self { store, storage, replicator, names, activity }
    }

    pub fn run(&self, from: &RelativePath, to: &RelativePath) -> OperationOutcome {
        let Some(library) = self.store.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        if let Some(violation) = FileNamePolicy::check(to.file_name()) {
            return OperationOutcome::InvalidName(violation);
        }
        // Names first: a clash with an existing folder is a taken name, whether or not it holds documents.
        match self.names.is_free(&library, to, Some(from)) {
            Err(error) => return OperationOutcome::StorageFailed(error),
            Ok(false) => return OperationOutcome::NameTaken,
            Ok(true) => {}
        }
        if let CatalogUpdate::Rejected(reason) = library.catalog.move_folder(from, to) {
            return OperationOutcome::Rejected(reason);
        }
        let replication = self.replicator.replicate(
            &library.folders,
            |primary| self.storage.move_entry(primary, from, to),
            &FolderOperation::Move { from: from.clone(), to: to.clone() },
        );
        let pending = match replication {
            Replication::PrimaryFailed(error) => return OperationOutcome::StorageFailed(error),
            Replication::Done { pending_folders, .. } => pending_folders,
        };
        let outcome = self.store.commit(|catalog| Ok(catalog.move_folder(from, to))).with_pending_folders(&pending);
        if matches!(outcome, OperationOutcome::Done { .. }) {
            self.activity.record(ActivityKind::FolderMoved, to.value());
        }
        outcome
    }
}
