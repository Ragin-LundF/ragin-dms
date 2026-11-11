use crate::DomainError;
use crate::error::require;
use crate::taxonomy::CategoryId;
use crate::text;

/// The category an AI suggests: one the catalog already has, or a new one the user may create.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SuggestedCategory {
    Existing(CategoryId),
    New { name: String, parent_id: Option<CategoryId> },
}

impl SuggestedCategory {
    /// # Errors
    /// Fails for a new category with a blank name.
    pub fn validate(&self) -> Result<(), DomainError> {
        match self {
            Self::Existing(_) => Ok(()),
            Self::New { name, .. } => {
                require(!text::is_blank(name), || "Suggested category name must not be blank".to_owned())
            }
        }
    }
}
