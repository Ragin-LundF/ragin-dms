use dms_domain::library::LibraryFolder;

use crate::port::plan::RestructureRecord;
use crate::port::storage::StorageResult;

/// Applied restructures in `.dms-plans/<name>.yaml` of a library folder, replicated like the text store.
pub trait PlanStore: Send + Sync {
    /// # Errors
    /// Fails when the record cannot be written.
    fn write(&self, folder: &LibraryFolder, record: &RestructureRecord) -> StorageResult<()>;

    /// The record called `name`, or `None` if there is none or it cannot be read.
    fn read(&self, folder: &LibraryFolder, name: &str) -> Option<RestructureRecord>;

    /// The names of the stored records, newest first.
    fn names(&self, folder: &LibraryFolder) -> Vec<String>;
}
