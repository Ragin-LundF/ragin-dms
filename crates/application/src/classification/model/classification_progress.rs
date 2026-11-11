use dms_domain::document::DocumentId;

use crate::classification::SuggestOutcome;

/// Progress of a batch classification: `done` of `total` documents, with the outcome of the last one.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassificationProgress {
    pub done: usize,
    pub total: usize,
    pub document_id: DocumentId,
    pub outcome: SuggestOutcome,
}
