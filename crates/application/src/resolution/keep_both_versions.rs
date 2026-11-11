use std::sync::Arc;

use dms_domain::library::{FileNamePolicy, LibraryFolderId, RelativePath};

use crate::common::{ActivityRecorder, OperationOutcome, SiblingNames};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;
use crate::port::storage::{DocumentStorage, FolderOperation};
use crate::sync::Replicator;

/// Keeps two versions of a path: the copy in one folder is renamed to a free name with a ` (2)` suffix and copied
/// into every folder. The renamed file is not catalogued yet; the next scan offers to adopt it.
pub struct KeepBothVersions {
    store: Arc<CatalogStore>,
    storage: Arc<dyn DocumentStorage>,
    replicator: Arc<Replicator>,
    names: Arc<SiblingNames>,
    activity: Arc<ActivityRecorder>,
}

impl KeepBothVersions {
    pub fn new(
        store: Arc<CatalogStore>,
        storage: Arc<dyn DocumentStorage>,
        replicator: Arc<Replicator>,
        names: Arc<SiblingNames>,
        activity: Arc<ActivityRecorder>,
    ) -> Self {
        Self { store, storage, replicator, names, activity }
    }

    pub fn run(&self, path: &RelativePath, folder_id: &LibraryFolderId) -> OperationOutcome {
        let Some(library) = self.store.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        let Some(folder) = library.folders.find(folder_id) else {
            return OperationOutcome::NotFound;
        };
        let parent = path.parent().unwrap_or_else(RelativePath::root);
        let known = match self.names.of(&library, &parent) {
            Ok(known) => known,
            Err(error) => return OperationOutcome::StorageFailed(error),
        };
        let in_folders = library
            .folders
            .folders()
            .iter()
            .flat_map(|other| self.storage.list_names(other, &parent).unwrap_or_default());
        let siblings: Vec<String> = known.into_iter().chain(in_folders).collect();
        let new_path = match parent.resolve(&FileNamePolicy::next_free_name(path.file_name(), &siblings)) {
            Ok(new_path) => new_path,
            Err(error) => return OperationOutcome::InvalidInput(error),
        };
        if let Err(error) = self.storage.move_entry(folder, path, &new_path) {
            return OperationOutcome::StorageFailed(error);
        }
        let primary = library.folders.primary();
        if folder != primary
            && let Err(error) = self.storage.copy_file(folder, primary, &new_path, false)
        {
            return OperationOutcome::StorageFailed(error);
        }
        let targets: Vec<_> = library.folders.sync_folders().into_iter().filter(|other| *other != folder).collect();
        let pending = self.replicator.distribute(
            &library.folders,
            &targets,
            &FolderOperation::CopyFromPrimary { path: new_path.clone(), replace_existing: false },
        );
        self.activity.record(ActivityKind::ConflictResolved, new_path.value());
        OperationOutcome::Done { catalog: library.catalog.clone(), pending_folders: pending }
    }
}
