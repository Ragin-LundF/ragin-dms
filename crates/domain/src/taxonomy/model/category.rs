use crate::DomainError;
use crate::error::require;
use crate::taxonomy::CategoryId;
use crate::text;

/// What kind of document something is (invoice, contract, certificate, …). Categories form a tree through
/// [`Self::parent_id`]; a document has at most one category.
///
/// [`Self::retention_years`] is an opt-in retention period the user set for documents of this category and its
/// subcategories; nothing suggests one.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Category {
    pub id: CategoryId,
    pub name: String,
    pub parent_id: Option<CategoryId>,
    pub retention_years: Option<u32>,
}

impl Category {
    pub const MAX_RETENTION_YEARS: u32 = 100;

    /// # Errors
    /// Fails like [`Self::validate`].
    pub fn new(
        id: CategoryId,
        name: impl Into<String>,
        parent_id: Option<CategoryId>,
        retention_years: Option<u32>,
    ) -> Result<Self, DomainError> {
        let category = Self { id, name: name.into(), parent_id, retention_years };
        category.validate()?;
        Ok(category)
    }

    /// # Errors
    /// Fails for a blank name, a category that is its own parent, or retention outside 1..=100 years.
    pub fn validate(&self) -> Result<(), DomainError> {
        require(!text::is_blank(&self.name), || "Category name must not be blank".to_owned())?;
        require(self.parent_id.as_ref() != Some(&self.id), || "A category cannot be its own parent".to_owned())?;
        require(self.retention_years.is_none_or(|years| (1..=Self::MAX_RETENTION_YEARS).contains(&years)), || {
            format!("Retention must be between 1 and {} years", Self::MAX_RETENTION_YEARS)
        })
    }
}
