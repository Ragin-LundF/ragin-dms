use crate::port::health::HealthReport;
use crate::port::storage::StorageResult;

/// The results of the archive check on this device (`health.yaml`), one report for the library last checked.
pub trait HealthStore: Send + Sync {
    /// The stored report, or `None` before the first check.
    ///
    /// # Errors
    /// Fails when the file cannot be read.
    fn load(&self) -> StorageResult<Option<HealthReport>>;

    /// # Errors
    /// Fails when the file cannot be written.
    fn save(&self, report: &HealthReport) -> StorageResult<()>;
}
