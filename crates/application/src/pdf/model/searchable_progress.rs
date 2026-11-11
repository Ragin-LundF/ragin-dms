use dms_domain::document::DocumentId;

use crate::pdf::SearchableOutcome;

/// Progress of making many scans searchable: `done` of `total`, with the outcome of the last one.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchableProgress {
    pub done: usize,
    pub total: usize,
    pub document_id: DocumentId,
    pub outcome: SearchableOutcome,
}
