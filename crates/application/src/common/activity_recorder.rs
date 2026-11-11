use std::sync::Arc;

use crate::port::state::{ActivityEntry, ActivityKind, ActivityLog};
use crate::port::system::Clock;

/// Writes one line to the activity log, stamped with the current time.
pub struct ActivityRecorder {
    activity: Arc<dyn ActivityLog>,
    clock: Arc<dyn Clock>,
}

impl ActivityRecorder {
    pub fn new(activity: Arc<dyn ActivityLog>, clock: Arc<dyn Clock>) -> Self {
        Self { activity, clock }
    }

    /// Best effort: the change it describes is already done, so a log that cannot be written does not turn the
    /// action into a failure (Kotlin let the exception escape after the change; see open-questions.md).
    pub fn record(&self, kind: ActivityKind, subject: &str) {
        let entry = ActivityEntry { at: self.clock.now(), kind, subject: subject.to_owned() };
        let _ = self.activity.record(&entry);
    }
}
