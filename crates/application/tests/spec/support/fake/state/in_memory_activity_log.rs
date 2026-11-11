use dms_application::port::state::{ActivityEntry, ActivityLog};
use dms_application::port::storage::StorageResult;

use crate::support::Shared;

#[derive(Default)]
pub struct InMemoryActivityLog {
    /// Newest first.
    pub entries: Shared<Vec<ActivityEntry>>,
}

impl ActivityLog for InMemoryActivityLog {
    fn record(&self, entry: &ActivityEntry) -> StorageResult<()> {
        self.entries.update(|entries| entries.insert(0, entry.clone()));
        Ok(())
    }

    fn recent(&self, limit: usize) -> StorageResult<Vec<ActivityEntry>> {
        Ok(self.entries.lock().iter().take(limit).cloned().collect())
    }
}
