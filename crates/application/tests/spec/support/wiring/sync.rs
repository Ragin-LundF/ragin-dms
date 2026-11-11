//! Wiring of the `sync` use cases on `TestLibrary`.

use std::sync::Arc;

use dms_application::sync::{MergeAllFolders, RetryFailedOperations, RetryOutcome, RetryReport, SyncFromPrimary};

use crate::support::TestLibrary;

impl TestLibrary {
    pub fn sync_from_primary(&self) -> SyncFromPrimary {
        SyncFromPrimary::new(self.store.clone(), self.folders.clone(), self.replicator.clone(), self.activity.clone())
    }

    pub fn merge_all_folders(&self) -> MergeAllFolders {
        MergeAllFolders::new(
            self.store.clone(),
            self.folders.clone(),
            self.folders.clone(),
            self.replicator.clone(),
            Arc::new(self.open_library()),
            self.activity.clone(),
        )
    }

    pub fn retry_failed_operations(&self) -> RetryFailedOperations {
        RetryFailedOperations::new(self.store.clone(), self.journal.clone(), self.replicator.clone())
    }

    /// The report of `retry_failed_operations`; `None` while no library is open.
    pub fn retried(&self) -> Option<RetryReport> {
        match self.retry_failed_operations().run() {
            RetryOutcome::Retried(report) => Some(report),
            RetryOutcome::LibraryNotOpen => None,
        }
    }
}
