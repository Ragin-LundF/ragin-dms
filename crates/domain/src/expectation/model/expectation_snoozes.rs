use chrono::NaiveDate;
use indexmap::IndexMap;

use crate::task::TaskKey;

/// The local, per-device "Requested" answers to "expected but missing" tasks: each task is hidden until its day.
/// "Ignore once" needs no entry here: it dismisses the task, whose key holds the expected date, so the next
/// period's task comes back on its own.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExpectationSnoozes {
    pub snoozed_until: IndexMap<TaskKey, NaiveDate>,
}

impl ExpectationSnoozes {
    /// Days a "Requested" answer hides a task.
    pub const REQUESTED_DAYS: u64 = 14;

    /// `true` while the task of `key` is snoozed on `today`.
    #[must_use]
    pub fn hides(&self, key: &TaskKey, today: NaiveDate) -> bool {
        self.snoozed_until.get(key).is_some_and(|until| today < *until)
    }
}
