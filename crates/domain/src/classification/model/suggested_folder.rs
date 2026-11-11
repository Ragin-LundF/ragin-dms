use crate::DomainError;
use crate::error::require;
use crate::library::RelativePath;

/// Where the AI suggests filing a document: an existing folder, or a new one ([`Self::is_new`]) below an existing
/// one.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SuggestedFolder {
    pub path: RelativePath,
    pub is_new: bool,
}

impl SuggestedFolder {
    /// # Errors
    /// Fails for the root.
    pub fn new(path: RelativePath, is_new: bool) -> Result<Self, DomainError> {
        let folder = Self { path, is_new };
        folder.validate()?;
        Ok(folder)
    }

    /// # Errors
    /// Fails for the root.
    pub fn validate(&self) -> Result<(), DomainError> {
        require(!self.path.is_root(), || "A suggested folder needs a path".to_owned())
    }
}
