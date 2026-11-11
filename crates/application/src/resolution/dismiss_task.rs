use std::sync::Arc;

use dms_domain::task::TaskKey;

use crate::port::state::TaskStore;
use crate::port::storage::StorageResult;

/// Hides a task from the dashboard without changing anything. The same change found again stays hidden.
pub struct DismissTask {
    tasks: Arc<dyn TaskStore>,
}

impl DismissTask {
    pub fn new(tasks: Arc<dyn TaskStore>) -> Self {
        Self { tasks }
    }

    /// # Errors
    /// The task store could not be written (Kotlin let the adapter's exception escape).
    pub fn run(&self, key: &TaskKey) -> StorageResult<()> {
        self.tasks.dismiss(key)
    }
}
