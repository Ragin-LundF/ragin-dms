use crate::DomainError;
use crate::library::{FileNamePolicy, FileNameViolation, RelativePath};

/// The optional inbox: a folder inside the library, at the same relative [`Self::path`] in every library folder,
/// where new documents are dropped and then filed. [`Self::ai_analysis`] lets the AI suggest metadata for inbox
/// documents, [`Self::ai_target_folder`] lets it also suggest where to file them, and [`Self::auto_analyze`] runs
/// the analysis as soon as a file arrives.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(clippy::struct_excessive_bools)] // independent switches of the catalog format
pub struct InboxSettings {
    pub enabled: bool,
    pub path: RelativePath,
    pub ai_analysis: bool,
    pub ai_target_folder: bool,
    pub auto_analyze: bool,
}

impl Default for InboxSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            path: Self::default_path(),
            ai_analysis: true,
            ai_target_folder: true,
            auto_analyze: false,
        }
    }
}

impl InboxSettings {
    /// `Inbox`.
    ///
    /// # Panics
    /// Never: the default path is valid.
    #[must_use]
    pub fn default_path() -> RelativePath {
        RelativePath::of("Inbox").expect("invariant: the default inbox path is valid")
    }

    /// # Errors
    /// Fails when [`Self::path_violation`] finds a problem with the path.
    pub fn validate(&self) -> Result<(), DomainError> {
        match Self::path_violation(&self.path) {
            Some(_) => Err(DomainError::InvalidArgument(format!("Invalid inbox path: {}", self.path))),
            None => Ok(()),
        }
    }

    /// `true` if `candidate` lies in the inbox, while it is switched on.
    #[must_use]
    pub fn contains(&self, candidate: &RelativePath) -> bool {
        self.enabled && candidate.is_inside(&self.path)
    }

    /// Why `path` cannot be the inbox, or `None` if it can: not the library root itself, and every folder name
    /// valid by [`FileNamePolicy`], which also keeps it out of the hidden `.dms-*` folders.
    #[must_use]
    pub fn path_violation(path: &RelativePath) -> Option<FileNameViolation> {
        if path.is_root() {
            return Some(FileNameViolation::Empty);
        }
        path.segments().into_iter().find_map(FileNamePolicy::check)
    }
}
