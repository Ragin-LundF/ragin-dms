use chrono::NaiveDate;
use dms_domain::catalog::CatalogMergeConflict;
use dms_domain::classification::ClassificationSuggestion;
use dms_domain::document::DocumentId;
use dms_domain::expectation::ExpectationSnoozes;
use dms_domain::task::TaskKey;
use indexmap::IndexSet;

use crate::port::storage::StorageResult;

/// Local state behind the dashboard that is not part of the catalog: pending AI suggestions, the conflicts of the
/// last catalog merge, tasks the user dismissed, and "expected but missing" tasks snoozed until a day.
///
/// The Kotlin functions threw when the state could not be read or written; here that is an error value.
pub trait TaskStore: Send + Sync {
    /// # Errors
    /// Fails when the state cannot be written.
    fn save_suggestion(&self, suggestion: &ClassificationSuggestion) -> StorageResult<()>;

    /// # Errors
    /// Fails when the state cannot be read.
    fn suggestions(&self) -> StorageResult<Vec<ClassificationSuggestion>>;

    /// # Errors
    /// Fails when the state cannot be written.
    fn remove_suggestion(&self, document_id: &DocumentId) -> StorageResult<()>;

    /// # Errors
    /// Fails when the state cannot be written.
    fn replace_merge_conflicts(&self, conflicts: &[CatalogMergeConflict]) -> StorageResult<()>;

    /// # Errors
    /// Fails when the state cannot be read.
    fn merge_conflicts(&self) -> StorageResult<Vec<CatalogMergeConflict>>;

    /// # Errors
    /// Fails when the state cannot be written.
    fn dismiss(&self, key: &TaskKey) -> StorageResult<()>;

    /// # Errors
    /// Fails when the state cannot be read.
    fn dismissed_keys(&self) -> StorageResult<IndexSet<TaskKey>>;

    /// Hides the task of `key` until `until`.
    ///
    /// # Errors
    /// Fails when the state cannot be written.
    fn snooze(&self, key: &TaskKey, until: NaiveDate) -> StorageResult<()>;

    /// # Errors
    /// Fails when the state cannot be read.
    fn snoozes(&self) -> StorageResult<ExpectationSnoozes>;
}
