use chrono::NaiveDate;

use crate::catalog::Catalog;
use crate::expectation::{ExpectationSnoozes, RhythmDetector};
use crate::task::OpenTask;

/// The dashboard tasks of documents that are expected but missing, minus the ones snoozed on this device.
pub struct ExpectationPlanner;

impl ExpectationPlanner {
    #[must_use]
    pub fn due_tasks(catalog: &Catalog, today: NaiveDate, snoozes: &ExpectationSnoozes) -> Vec<OpenTask> {
        RhythmDetector::missing(catalog, today)
            .into_iter()
            .map(OpenTask::ExpectedDocumentMissing)
            .filter(|task| !snoozes.hides(&task.key(), today))
            .collect()
    }
}
