use std::sync::Arc;

use dms_domain::document::DocumentId;

use crate::port::state::TaskStore;
use crate::port::storage::StorageResult;

/// Discards the pending suggestion of a document. The document stays as it is.
pub struct RejectSuggestion {
    tasks: Arc<dyn TaskStore>,
}

impl RejectSuggestion {
    pub fn new(tasks: Arc<dyn TaskStore>) -> Self {
        Self { tasks }
    }

    /// # Errors
    /// The task store could not be written (Kotlin threw).
    pub fn run(&self, document_id: &DocumentId) -> StorageResult<()> {
        self.tasks.remove_suggestion(document_id)
    }
}
