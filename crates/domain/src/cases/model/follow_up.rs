use chrono::NaiveDate;

use crate::DomainError;
use crate::cases::{FollowUpId, FollowUpSource};
use crate::error::require_not_blank;

/// Something to do in a case by `date`, such as sending documents or replying.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FollowUp {
    pub id: FollowUpId,
    pub date: NaiveDate,
    pub text: String,
    pub done: bool,
    pub source: FollowUpSource,
}

impl FollowUp {
    /// # Errors
    /// Fails for a blank text.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_not_blank(&self.text, "Follow-up")
    }
}
