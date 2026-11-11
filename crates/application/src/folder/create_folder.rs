use std::sync::Arc;

use dms_domain::library::{FileNamePolicy, RelativePath};

use crate::common::{ActivityRecorder, OperationOutcome, SiblingNames};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;
use crate::port::storage::{DocumentStorage, FolderOperation};
use crate::sync::{Replication, Replicator};

/// Creates an empty folder in every library folder. The catalog does not change.
pub struct CreateFolder {
    store: Arc<CatalogStore>,
    storage: Arc<dyn DocumentStorage>,
    replicator: Arc<Replicator>,
    names: Arc<SiblingNames>,
    activity: Arc<ActivityRecorder>,
}

impl CreateFolder {
    pub fn new(
        store: Arc<CatalogStore>,
        storage: Arc<dyn DocumentStorage>,
        replicator: Arc<Replicator>,
        names: Arc<SiblingNames>,
        activity: Arc<ActivityRecorder>,
    ) -> Self {
        Self { store, storage, replicator, names, activity }
    }

    pub fn run(&self, path: &RelativePath) -> OperationOutcome {
        let Some(library) = self.store.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        if let Some(violation) = FileNamePolicy::check(path.file_name()) {
            return OperationOutcome::InvalidName(violation);
        }
        match self.names.is_free(&library, path, None) {
            Err(error) => return OperationOutcome::StorageFailed(error),
            Ok(false) => return OperationOutcome::NameTaken,
            Ok(true) => {}
        }
        let replication = self.replicator.replicate(
            &library.folders,
            |primary| self.storage.create_folder(primary, path),
            &FolderOperation::CreateFolder(path.clone()),
        );
        match replication {
            Replication::PrimaryFailed(error) => OperationOutcome::StorageFailed(error),
            Replication::Done { pending_folders, .. } => {
                self.activity.record(ActivityKind::FolderCreated, path.value());
                OperationOutcome::Done { catalog: library.catalog, pending_folders }
            }
        }
    }
}
