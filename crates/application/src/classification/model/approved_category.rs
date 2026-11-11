use dms_domain::DomainError;
use dms_domain::taxonomy::CategoryId;
use dms_domain::text;

/// The category the user approved: one the catalog has, or a path of names from the top level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovedCategory {
    Existing(CategoryId),
    /// A category by its names from the top level down, like `Insurance / Household / Rent`. A level whose name
    /// already exists at the same place is reused; every other level is created.
    Path(Vec<String>),
}

impl ApprovedCategory {
    /// # Errors
    /// Fails for an empty path or a blank name.
    pub fn path(names: Vec<String>) -> Result<Self, DomainError> {
        let category = Self::Path(names);
        category.validate()?;
        Ok(category)
    }

    /// # Errors
    /// See [`Self::path`].
    pub fn validate(&self) -> Result<(), DomainError> {
        let Self::Path(names) = self else {
            return Ok(());
        };
        if names.is_empty() {
            return Err(DomainError::InvalidArgument("A category path needs at least one name".to_owned()));
        }
        if names.iter().any(|name| text::is_blank(name)) {
            return Err(DomainError::InvalidArgument("A category name must not be blank".to_owned()));
        }
        Ok(())
    }
}
