use std::sync::Arc;

use dms_domain::library::{LibraryFolderId, LibraryFolders};
use indexmap::IndexMap;

use crate::library::CatalogStore;
use crate::port::storage::{OperationJournal, PendingOperation, StorageFailure, StorageResult};
use crate::sync::{Replicator, RetryOutcome, RetryReport};

/// The source is gone, or the target already holds something else at that path: the operation no longer applies,
/// and the next scan shows any difference that is left.
const OBSOLETE: [StorageFailure; 2] = [StorageFailure::NotFound, StorageFailure::AlreadyExists];

/// Replays the journal: the operations of each sync folder in the order they were recorded. The first operation
/// that still fails stops that folder, so later operations never overtake earlier ones. Operations that no longer
/// apply are dropped. A journal that cannot be read counts as empty; a failed removal is dropped (Kotlin threw).
pub struct RetryFailedOperations {
    store: Arc<CatalogStore>,
    journal: Arc<dyn OperationJournal>,
    replicator: Arc<Replicator>,
}

impl RetryFailedOperations {
    pub fn new(store: Arc<CatalogStore>, journal: Arc<dyn OperationJournal>, replicator: Arc<Replicator>) -> Self {
        Self { store, journal, replicator }
    }

    pub fn run(&self) -> RetryOutcome {
        let Some(library) = self.store.current() else {
            return RetryOutcome::LibraryNotOpen;
        };
        let mut pending = self.journal.pending().unwrap_or_default();
        pending.sort_by_key(|operation| operation.recorded_at);
        let mut by_folder: IndexMap<LibraryFolderId, Vec<PendingOperation>> = IndexMap::new();
        for operation in pending {
            by_folder.entry(operation.folder_id.clone()).or_default().push(operation);
        }
        let report = by_folder
            .iter()
            .map(|(folder_id, operations)| self.retry(&library.folders, folder_id, operations))
            .fold(RetryReport::default(), |total, report| total + report);
        RetryOutcome::Retried(report)
    }

    fn retry(
        &self,
        folders: &LibraryFolders,
        folder_id: &LibraryFolderId,
        operations: &[PendingOperation],
    ) -> RetryReport {
        let Some(target) = folders.sync_folders().into_iter().find(|folder| &folder.id == folder_id) else {
            // The folder was removed or became the primary; its operations no longer apply.
            for operation in operations {
                let _ = self.journal.remove(&operation.id);
            }
            return RetryReport { dropped: operations.len(), ..RetryReport::default() };
        };
        let mut report = RetryReport::default();
        for (index, pending) in operations.iter().enumerate() {
            let result = self.replicator.execute(folders.primary(), target, &pending.operation);
            let Some(step) = self.outcome(pending, &result) else {
                return report + RetryReport { still_pending: operations.len() - index, ..RetryReport::default() };
            };
            report = report + step;
        }
        report
    }

    /// Removes a finished or pointless operation from the journal; `None` if it must stay.
    fn outcome(&self, pending: &PendingOperation, result: &StorageResult<()>) -> Option<RetryReport> {
        match result {
            Ok(()) => {
                let _ = self.journal.remove(&pending.id);
                Some(RetryReport { succeeded: 1, ..RetryReport::default() })
            }
            Err(error) if OBSOLETE.contains(&error.reason) => {
                let _ = self.journal.remove(&pending.id);
                Some(RetryReport { dropped: 1, ..RetryReport::default() })
            }
            Err(_) => None,
        }
    }
}
