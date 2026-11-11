use chrono::NaiveDate;

use crate::DomainError;
use crate::error::require;
use crate::text;

/// Descriptive data about a document, edited by the user or filled from an approved suggestion.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub document_date: Option<NaiveDate>,
    pub correspondent: Option<String>,
}

impl DocumentMetadata {
    /// # Errors
    /// Fails when title or correspondent is given but blank.
    pub fn new(
        title: Option<String>,
        document_date: Option<NaiveDate>,
        correspondent: Option<String>,
    ) -> Result<Self, DomainError> {
        let metadata = Self { title, document_date, correspondent };
        metadata.validate()?;
        Ok(metadata)
    }

    /// # Errors
    /// Fails when title or correspondent is given but blank.
    pub fn validate(&self) -> Result<(), DomainError> {
        require(self.title.as_deref().is_none_or(|title| !text::is_blank(title)), || {
            "Title must be null or not blank".to_owned()
        })?;
        require(self.correspondent.as_deref().is_none_or(|name| !text::is_blank(name)), || {
            "Correspondent must be null or not blank".to_owned()
        })
    }
}
