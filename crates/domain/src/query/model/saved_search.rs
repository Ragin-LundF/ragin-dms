use crate::DomainError;
use crate::error::require;
use crate::query::{DocumentQuery, SavedSearchId};
use crate::text;

/// A named [`DocumentQuery`], shown as a smart folder and usable as the scope of restructure and export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedSearch {
    pub id: SavedSearchId,
    pub name: String,
    pub query: DocumentQuery,
}

impl SavedSearch {
    /// # Errors
    /// Fails for a blank name or an invalid query.
    pub fn new(id: SavedSearchId, name: impl Into<String>, query: DocumentQuery) -> Result<Self, DomainError> {
        let search = Self { id, name: name.into(), query };
        search.validate()?;
        Ok(search)
    }

    /// # Errors
    /// Fails for a blank name or an invalid query.
    pub fn validate(&self) -> Result<(), DomainError> {
        require(!text::is_blank(&self.name), || "Saved search name must not be blank".to_owned())?;
        self.query.validate()
    }
}
