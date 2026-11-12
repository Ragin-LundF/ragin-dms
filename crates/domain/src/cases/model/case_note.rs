use chrono::NaiveDate;

use crate::DomainError;
use crate::error::require_not_blank;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CaseNote {
    pub date: NaiveDate,
    pub text: String,
}

impl CaseNote {
    /// # Errors
    /// Fails for a blank text.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_not_blank(&self.text, "Note")
    }
}
