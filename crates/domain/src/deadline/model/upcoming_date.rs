use chrono::NaiveDate;

use crate::document::DocumentId;
use crate::field::FieldKey;

/// A date ahead for one document: a deadline [`Self::field`], or its retention end when [`Self::field`] is `None`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UpcomingDate {
    pub document_id: DocumentId,
    pub field: Option<FieldKey>,
    pub date: NaiveDate,
}
