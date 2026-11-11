use std::sync::Arc;

use dms_domain::change::NewFile;
use indexmap::IndexSet;

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;
use crate::port::storage::DocumentStorage;

/// Moves a file the catalog does not know to the system trash in every folder that has it, because it is a copy
/// of a catalogued document. Safe by design: the catalog must know a document with the same size and SHA-256, and
/// in each folder the two files are compared byte by byte right before the copy goes. A folder where they are not
/// identical, or where the document itself is missing, keeps the file. Only with the user's confirmation; the
/// catalog does not change.
pub struct TrashDuplicateFile {
    store: Arc<CatalogStore>,
    storage: Arc<dyn DocumentStorage>,
    activity: Arc<ActivityRecorder>,
}

impl TrashDuplicateFile {
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
        let Some(content_hash) = &change.content_hash else {
            return OperationOutcome::ScanOutdated;
        };
        if library.catalog.find_by_path(&change.path).is_some() {
            return OperationOutcome::ScanOutdated;
        }
        let Some(original) = library
            .catalog
            .find_by_content_hash(content_hash)
            .into_iter()
            .find(|document| document.size == change.size)
        else {
            return OperationOutcome::NotFound;
        };
        let trashed = library
            .folders
            .folders()
            .iter()
            .filter(|folder| change.present_in.contains(&folder.id))
            .filter(|folder| {
                self.storage.same_content(folder, &original.path, &change.path) == Ok(true)
                    && self.storage.move_to_trash(folder, &change.path).is_ok()
            })
            .count();
        if trashed == 0 {
            return OperationOutcome::ScanOutdated;
        }
        self.activity.record(ActivityKind::DocumentTrashed, change.path.value());
        OperationOutcome::Done { catalog: library.catalog.clone(), pending_folders: IndexSet::new() }
    }
}
