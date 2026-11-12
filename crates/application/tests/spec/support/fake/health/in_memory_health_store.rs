use dms_application::port::health::{HealthReport, HealthStore};
use dms_application::port::storage::StorageResult;

use crate::support::Shared;

#[derive(Default)]
pub struct InMemoryHealthStore {
    pub report: Shared<Option<HealthReport>>,
    /// Every saved report, so a test sees the resume points a pass wrote.
    pub saves: Shared<usize>,
}

impl HealthStore for InMemoryHealthStore {
    fn load(&self) -> StorageResult<Option<HealthReport>> {
        Ok(self.report.lock().clone())
    }

    fn save(&self, report: &HealthReport) -> StorageResult<()> {
        self.report.set(Some(report.clone()));
        self.saves.update(|saves| *saves += 1);
        Ok(())
    }
}
