use dms_domain::DomainError;
use dms_domain::text;

/// Where an export goes on this device: a new folder [`Self::name`] in [`Self::directory`], or `name.zip` there
/// with [`Self::zip`]. An existing folder or file of that name is never replaced; the writer picks a free name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExportTarget {
    pub directory: String,
    pub name: String,
    pub zip: bool,
}

impl ExportTarget {
    /// # Errors
    /// Fails for a blank directory or name.
    pub fn new(directory: impl Into<String>, name: impl Into<String>, zip: bool) -> Result<Self, DomainError> {
        let target = Self { directory: directory.into(), name: name.into(), zip };
        target.validate()?;
        Ok(target)
    }

    /// # Errors
    /// Fails for a blank directory or name.
    pub fn validate(&self) -> Result<(), DomainError> {
        if text::is_blank(&self.directory) {
            return Err(DomainError::InvalidArgument("Export directory must not be blank".to_owned()));
        }
        if text::is_blank(&self.name) {
            return Err(DomainError::InvalidArgument("Export name must not be blank".to_owned()));
        }
        Ok(())
    }
}
