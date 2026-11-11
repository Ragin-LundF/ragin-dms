use chrono::{DateTime, NaiveDate, Utc};

/// How long the user wants to keep one document. Strictly opt-in: a document has no retention unless the user
/// sets [`Self::keep_until`] here or a period on its category. Once the user marked it done
/// ([`Self::settled_at`]), the document is never flagged again, also not through its category.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Retention {
    pub keep_until: Option<NaiveDate>,
    pub settled_at: Option<DateTime<Utc>>,
}

impl Retention {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keep_until.is_none() && self.settled_at.is_none()
    }
}
