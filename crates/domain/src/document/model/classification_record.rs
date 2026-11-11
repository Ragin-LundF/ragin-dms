use chrono::{DateTime, Utc};

use crate::DomainError;
use crate::document::ClassificationSource;
use crate::error::require;
use crate::text;

/// Provenance of a document's categorization: who suggested it, with which model, confirmed when.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClassificationRecord {
    pub source: ClassificationSource,
    pub model: Option<String>,
    pub confirmed_at: DateTime<Utc>,
}

impl ClassificationRecord {
    /// # Errors
    /// Fails for an AI classification without a model name.
    pub fn new(
        source: ClassificationSource,
        model: Option<String>,
        confirmed_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        let record = Self { source, model, confirmed_at };
        record.validate()?;
        Ok(record)
    }

    /// # Errors
    /// Fails for an AI classification without a model name.
    pub fn validate(&self) -> Result<(), DomainError> {
        let named = self.model.as_deref().is_some_and(|model| !text::is_blank(model));
        require(self.source != ClassificationSource::Ai || named, || {
            "An AI classification must name its model".to_owned()
        })
    }
}
