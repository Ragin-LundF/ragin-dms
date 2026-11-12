use chrono::NaiveDate;
use dms_application::port::state::TaskStore;
use dms_application::port::storage::StorageResult;
use dms_domain::catalog::CatalogMergeConflict;
use dms_domain::classification::ClassificationSuggestion;
use dms_domain::document::DocumentId;
use dms_domain::expectation::ExpectationSnoozes;
use dms_domain::task::TaskKey;
use indexmap::{IndexMap, IndexSet};

use crate::support::Shared;

#[derive(Default)]
pub struct InMemoryTaskStore {
    pub suggestions: Shared<IndexMap<DocumentId, ClassificationSuggestion>>,
    pub conflicts: Shared<Vec<CatalogMergeConflict>>,
    pub dismissed: Shared<IndexSet<TaskKey>>,
    pub snoozes: Shared<ExpectationSnoozes>,
}

impl TaskStore for InMemoryTaskStore {
    fn save_suggestion(&self, suggestion: &ClassificationSuggestion) -> StorageResult<()> {
        self.suggestions.update(|suggestions| suggestions.insert(suggestion.document_id.clone(), suggestion.clone()));
        Ok(())
    }

    fn suggestions(&self) -> StorageResult<Vec<ClassificationSuggestion>> {
        Ok(self.suggestions.lock().values().cloned().collect())
    }

    fn remove_suggestion(&self, document_id: &DocumentId) -> StorageResult<()> {
        self.suggestions.update(|suggestions| suggestions.shift_remove(document_id));
        Ok(())
    }

    fn replace_merge_conflicts(&self, conflicts: &[CatalogMergeConflict]) -> StorageResult<()> {
        self.conflicts.set(conflicts.to_vec());
        Ok(())
    }

    fn merge_conflicts(&self) -> StorageResult<Vec<CatalogMergeConflict>> {
        Ok(self.conflicts.get())
    }

    fn dismiss(&self, key: &TaskKey) -> StorageResult<()> {
        self.dismissed.update(|dismissed| dismissed.insert(key.clone()));
        Ok(())
    }

    fn dismissed_keys(&self) -> StorageResult<IndexSet<TaskKey>> {
        Ok(self.dismissed.get())
    }

    fn snooze(&self, key: &TaskKey, until: NaiveDate) -> StorageResult<()> {
        self.snoozes.update(|snoozes| snoozes.snoozed_until.insert(key.clone(), until));
        Ok(())
    }

    fn snoozes(&self) -> StorageResult<ExpectationSnoozes> {
        Ok(self.snoozes.get())
    }
}
