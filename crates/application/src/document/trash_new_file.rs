use std::sync::Arc;

use dms_domain::change::NewFile;
use indexmap::IndexSet;

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;
use crate::port::storage::DocumentStorage;

/// Moves a file the catalog does not know to the system trash in every folder that has it: the other answer to a
/// file found on one side only, besides adopting it. Only with the user's confirmation; the catalog does not
/// change.
pub struct TrashNewFile {
    store: Arc<CatalogStore>,
    storage: Arc<dyn DocumentStorage>,
    activity: Arc<ActivityRecorder>,
}

impl TrashNewFile {
    pub fn new(store: Arc<CatalogStore>, storage: Arc<dyn DocumentStorage>, activity: Arc<ActivityRecorder>) -> Self {
        Self { store, storage, activity }
    }

    pub fn run(&self, change: &NewFile, confirmed: bool) -> OperationOutcome {
        if !confirmed {
            return OperationOutcome::NotConfirmed;
        }
        let Some(library) = self.store.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        if library.catalog.find_by_path(&change.path).is_some() {
            return OperationOutcome::ScanOutdated;
        }
        for folder in library.folders.folders().iter().filter(|folder| change.present_in.contains(&folder.id)) {
            if let Err(error) = self.storage.move_to_trash(folder, &change.path) {
                return OperationOutcome::StorageFailed(error);
            }
        }
        self.activity.record(ActivityKind::DocumentTrashed, change.path.value());
        OperationOutcome::Done { catalog: library.catalog, pending_folders: IndexSet::new() }
    }
}
