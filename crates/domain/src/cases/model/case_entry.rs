use chrono::NaiveDate;

use crate::cases::EntryDirection;
use crate::document::DocumentId;

/// A document of a case: received or sent on `date`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CaseEntry {
    pub document_id: DocumentId,
    pub direction: EntryDirection,
    pub date: NaiveDate,
}
