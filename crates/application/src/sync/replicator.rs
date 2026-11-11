use std::sync::Arc;

use dms_domain::catalog::{Catalog, CatalogRevision};
use dms_domain::document::DocumentId;
use dms_domain::library::{LibraryFolder, LibraryFolderId, LibraryFolders, RelativePath};
use indexmap::IndexSet;

use crate::port::index::VectorIndex;
use crate::port::plan::PlanStore;
use crate::port::storage::{
    CatalogLoad, CatalogRepository, DocumentStorage, FolderOperation, OperationJournal, PendingOperation, StorageError,
    StorageFailure, StorageResult, TextStore,
};
use crate::port::system::{Clock, IdGenerator};
use crate::sync::Replication;

/// Applies changes to every library folder: the primary first, then each sync folder. A failed primary aborts
/// before any sync folder is touched; a failed sync folder is recorded in the journal and the others continue.
pub struct Replicator {
    storage: Arc<dyn DocumentStorage>,
    catalogs: Arc<dyn CatalogRepository>,
    texts: Arc<dyn TextStore>,
    plans: Arc<dyn PlanStore>,
    index: Arc<dyn VectorIndex>,
    journal: Arc<dyn OperationJournal>,
    clock: Arc<dyn Clock>,
    ids: Arc<dyn IdGenerator>,
}

impl Replicator {
    #[allow(clippy::too_many_arguments)] // one port each, as in Kotlin
    pub fn new(
        storage: Arc<dyn DocumentStorage>,
        catalogs: Arc<dyn CatalogRepository>,
        texts: Arc<dyn TextStore>,
        plans: Arc<dyn PlanStore>,
        index: Arc<dyn VectorIndex>,
        journal: Arc<dyn OperationJournal>,
        clock: Arc<dyn Clock>,
        ids: Arc<dyn IdGenerator>,
    ) -> Self {
        Self { storage, catalogs, texts, plans, index, journal, clock, ids }
    }

    /// Runs `on_primary` on the primary, then brings every sync folder in line with `follow_up`.
    pub fn replicate<T>(
        &self,
        folders: &LibraryFolders,
        on_primary: impl FnOnce(&LibraryFolder) -> StorageResult<T>,
        follow_up: &FolderOperation,
    ) -> Replication<T> {
        match on_primary(folders.primary()) {
            Err(error) => Replication::PrimaryFailed(error),
            Ok(value) => Replication::Done {
                value,
                pending_folders: self.distribute(folders, &folders.sync_folders(), follow_up),
            },
        }
    }

    /// Saves `catalog` to every folder. `based_on` is the newest revision the change was made on; a primary that
    /// holds a newer one fails with [`StorageFailure::CatalogChanged`] and nothing is written.
    pub fn save_catalog(
        &self,
        folders: &LibraryFolders,
        catalog: &Catalog,
        based_on: CatalogRevision,
    ) -> Replication<()> {
        self.replicate(folders, |primary| self.catalogs.save(primary, catalog, based_on), &FolderOperation::SaveCatalog)
    }

    /// Moves `path` to the trash in every folder. A folder where it is already gone counts as done.
    pub fn trash(&self, folders: &LibraryFolders, path: &RelativePath) -> Replication<()> {
        self.replicate(
            folders,
            |primary| or_if_missing(self.storage.move_to_trash(primary, path)),
            &FolderOperation::MoveToTrash(path.clone()),
        )
    }

    /// Applies `operation` to each of `targets` and returns the ids of the folders where it failed. Failures are
    /// recorded in the journal.
    pub fn distribute(
        &self,
        folders: &LibraryFolders,
        targets: &[&LibraryFolder],
        operation: &FolderOperation,
    ) -> IndexSet<LibraryFolderId> {
        targets
            .iter()
            .filter(|target| {
                let failed = self.execute(folders.primary(), target, operation).is_err();
                if failed {
                    self.record(&target.id, operation);
                }
                failed
            })
            .map(|target| target.id.clone())
            .collect()
    }

    /// Executes one operation against one sync folder, reading whatever it needs from the primary.
    ///
    /// # Errors
    /// The operation failed in `target`, or what it copies is missing in `primary`.
    pub fn execute(
        &self,
        primary: &LibraryFolder,
        target: &LibraryFolder,
        operation: &FolderOperation,
    ) -> StorageResult<()> {
        match operation {
            FolderOperation::CopyFromPrimary { path, replace_existing } => {
                self.copy(primary, target, path, *replace_existing)
            }
            FolderOperation::CreateFolder(path) => self.storage.create_folder(target, path),
            FolderOperation::Move { from, to } => self.storage.move_entry(target, from, to),
            FolderOperation::MoveToTrash(path) => or_if_missing(self.storage.move_to_trash(target, path)),
            FolderOperation::CopyText(document_id) => self.copy_text(primary, target, document_id),
            FolderOperation::CopyPlan(name) => self.copy_plan(primary, target, name),
            FolderOperation::CopyIndex(document_id) => self.copy_index(primary, target, document_id),
            FolderOperation::SaveCatalog => self.copy_catalog(primary, target),
        }
    }

    /// A copy that finds the same content already in place has reached its goal.
    fn copy(
        &self,
        primary: &LibraryFolder,
        target: &LibraryFolder,
        path: &RelativePath,
        replace_existing: bool,
    ) -> StorageResult<()> {
        let copied = self.storage.copy_file(primary, target, path, replace_existing).map(|_| ());
        if let Err(error) = &copied
            && error.reason == StorageFailure::AlreadyExists
        {
            let source = self.storage.read_facts(primary, path);
            let existing = self.storage.read_facts(target, path);
            if source.is_ok() && source == existing {
                return Ok(());
            }
        }
        copied
    }

    fn copy_text(
        &self,
        primary: &LibraryFolder,
        target: &LibraryFolder,
        document_id: &DocumentId,
    ) -> StorageResult<()> {
        let text = self.texts.read(primary, document_id).ok_or_else(not_found)?;
        self.texts.write(target, document_id, &text)
    }

    fn copy_plan(&self, primary: &LibraryFolder, target: &LibraryFolder, name: &str) -> StorageResult<()> {
        let record = self.plans.read(primary, name).ok_or_else(not_found)?;
        self.plans.write(target, &record)
    }

    fn copy_index(
        &self,
        primary: &LibraryFolder,
        target: &LibraryFolder,
        document_id: &DocumentId,
    ) -> StorageResult<()> {
        let vectors = self.index.read(primary, document_id).ok_or_else(not_found)?;
        self.index.write(target, &vectors)
    }

    /// Never lowers a sync folder's revision: a sync folder holding a newer catalog fails with
    /// [`StorageFailure::CatalogChanged`] and stays in the journal until opening the library merged it.
    fn copy_catalog(&self, primary: &LibraryFolder, target: &LibraryFolder) -> StorageResult<()> {
        match self.catalogs.load(primary) {
            CatalogLoad::Found { catalog, .. } => self.catalogs.save(target, &catalog, catalog.revision()),
            CatalogLoad::NotFound => Err(not_found()),
            CatalogLoad::Failed(error) => Err(error),
        }
    }

    /// A journal that cannot be written loses the entry; the folder is still reported as pending, and the next
    /// scan shows what is missing (Kotlin let the exception escape; see open-questions.md).
    fn record(&self, folder_id: &LibraryFolderId, operation: &FolderOperation) {
        let pending = PendingOperation {
            id: self.ids.new_id(),
            folder_id: folder_id.clone(),
            operation: operation.clone(),
            recorded_at: self.clock.now(),
        };
        let _ = self.journal.record(&pending);
    }
}

fn not_found() -> StorageError {
    StorageError::new(StorageFailure::NotFound)
}

/// Trashing something that is already gone has reached its goal.
fn or_if_missing(result: StorageResult<()>) -> StorageResult<()> {
    match result {
        Err(error) if error.reason == StorageFailure::NotFound => Ok(()),
        other => other,
    }
}
