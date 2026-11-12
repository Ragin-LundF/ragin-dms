use std::sync::Arc;

use dms_domain::expectation::ExpectationSnoozes;
use dms_domain::task::TaskKey;

use crate::port::state::TaskStore;
use crate::port::storage::StorageResult;
use crate::port::system::Clock;

/// "Requested": the user asked the sender for the missing document; its task hides on this device for
/// [`ExpectationSnoozes::REQUESTED_DAYS`] days, then asks again.
pub struct SnoozeExpectedDocument {
    tasks: Arc<dyn TaskStore>,
    clock: Arc<dyn Clock>,
}

impl SnoozeExpectedDocument {
    pub fn new(tasks: Arc<dyn TaskStore>, clock: Arc<dyn Clock>) -> Self {
        Self { tasks, clock }
    }

    /// # Errors
    /// The task store could not be written.
    pub fn run(&self, key: &TaskKey) -> StorageResult<()> {
        let until = self.clock.today() + chrono::Days::new(ExpectationSnoozes::REQUESTED_DAYS);
        self.tasks.snooze(key, until)
    }
}
