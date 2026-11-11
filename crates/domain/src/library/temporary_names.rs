use crate::DomainError;
use crate::library::RelativePath;
use crate::text;

/// The names of the tool's unfinished writes. Every module that writes a temporary file uses them, so the
/// scanner skips them and the user can recognize and remove leftovers.
pub struct TemporaryNames;

impl TemporaryNames {
    pub const PREFIX: &'static str = ".dms-tmp-";

    #[must_use]
    pub fn is_temporary(name: &str) -> bool {
        name.starts_with(Self::PREFIX)
    }

    /// A temporary name next to `path`, made unique by `unique_id`. It keeps `path`'s extension, so tools that
    /// go by the extension still recognize the file.
    ///
    /// # Errors
    /// Fails when `unique_id` makes no valid path segment, for example with a `/`.
    pub fn sibling_of(path: &RelativePath, unique_id: &str) -> Result<RelativePath, DomainError> {
        let extension = text::after_last(path.file_name(), '.');
        let suffix = if extension.is_empty() { String::new() } else { format!(".{extension}") };
        path.parent().unwrap_or_default().resolve(&format!("{}{unique_id}{suffix}", Self::PREFIX))
    }
}
