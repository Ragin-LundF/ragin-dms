use std::sync::Arc;

use dms_domain::library::RelativePath;
use indexmap::IndexSet;

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;
use crate::port::storage::DocumentStorage;
use crate::sync::{Replication, Replicator};

/// Moves folders a restructure left without documents to the trash in every library folder, after the user
/// confirmed. A folder that holds a document again, or any visible file, is kept.
pub struct RemoveEmptyFolders {
    store: Arc<CatalogStore>,
    storage: Arc<dyn DocumentStorage>,
    replicator: Arc<Replicator>,
    activity: Arc<ActivityRecorder>,
}

impl RemoveEmptyFolders {
    pub fn new(
        store: Arc<CatalogStore>,
        storage: Arc<dyn DocumentStorage>,
        replicator: Arc<Replicator>,
        activity: Arc<ActivityRecorder>,
    ) -> Self {
        Self { store, storage, replicator, activity }
    }

    pub fn run(&self, folders: &[RelativePath], confirmed: bool) -> OperationOutcome {
        if !confirmed {
            return OperationOutcome::NotConfirmed;
        }
        let Some(library) = self.store.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        let mut pending = IndexSet::new();
        // Deepest first, so a parent is only checked after its emptied children are gone.
        let mut ordered: Vec<&RelativePath> = folders.iter().collect();
        ordered.sort_by_key(|folder| std::cmp::Reverse(folder.segments().len()));
        for folder in ordered {
            let catalogued = library.catalog.documents().values().any(|document| document.path.is_inside(folder));
            let empty = self
                .storage
                .list_names(library.folders.primary(), folder)
                .is_ok_and(|names| names.iter().all(|name| name.starts_with('.')));
            if folder.is_root() || catalogued || !empty {
                continue;
            }
            match self.replicator.trash(&library.folders, folder) {
                Replication::PrimaryFailed(error) => return OperationOutcome::StorageFailed(error),
                Replication::Done { pending_folders, .. } => {
                    pending.extend(pending_folders);
                    // Logged as a trashed document, as in Kotlin (parity-checklist open point 18).
                    self.activity.record(ActivityKind::DocumentTrashed, folder.value());
                }
            }
        }
        OperationOutcome::Done { catalog: library.catalog, pending_folders: pending }
    }
}
