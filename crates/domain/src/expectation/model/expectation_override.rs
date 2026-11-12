use crate::DomainError;
use crate::error::require_not_blank;
use crate::expectation::ExpectationMode;
use crate::taxonomy::CategoryId;
use crate::text;

/// The user's decision about the rhythm of one sender and category; the sender's name is compared ignoring case.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpectationOverride {
    pub correspondent: String,
    pub category_id: Option<CategoryId>,
    pub mode: ExpectationMode,
}

impl ExpectationOverride {
    /// # Errors
    /// Fails for a blank correspondent.
    pub fn validate(&self) -> Result<(), DomainError> {
        require_not_blank(&self.correspondent, "Correspondent")
    }

    /// The identity of the override: the sender's name in lower case and the category.
    #[must_use]
    pub fn key(&self) -> (String, Option<CategoryId>) {
        (text::lowercase(text::trim(&self.correspondent)), self.category_id.clone())
    }
}
