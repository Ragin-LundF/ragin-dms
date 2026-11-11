use std::fmt;

use dms_domain::DomainError;
use dms_domain::text;

/// A file outside the library that the user wants to add, for example one dropped from the file manager.
/// [`Self::location`] is an absolute path in the syntax of this device.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalFile(String);

impl ExternalFile {
    /// # Errors
    /// Fails for a blank location.
    pub fn new(location: impl Into<String>) -> Result<Self, DomainError> {
        let location = location.into();
        if text::is_blank(&location) {
            return Err(DomainError::InvalidArgument("Location must not be blank".to_owned()));
        }
        Ok(Self(location))
    }

    #[must_use]
    pub fn location(&self) -> &str {
        &self.0
    }

    /// The last path segment, for both `/` and `\` separated locations.
    #[must_use]
    pub fn file_name(&self) -> &str {
        let trimmed = self.0.trim_end_matches(['/', '\\']);
        let after_slash = trimmed.rsplit('/').next().unwrap_or_default();
        after_slash.rsplit('\\').next().unwrap_or_default()
    }
}

impl fmt::Display for ExternalFile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_file_name_is_the_last_segment_of_either_separator() {
        let name = |location: &str| ExternalFile::new(location).expect("valid").file_name().to_owned();

        assert_eq!(name("/home/user/scan.pdf"), "scan.pdf");
        assert_eq!(name("C:\\Users\\scan.pdf"), "scan.pdf");
        assert_eq!(name("/home/user/folder/"), "folder");
        assert!(ExternalFile::new("  ").is_err());
    }
}
