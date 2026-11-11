use dms_domain::library::RelativePath;
use indexmap::IndexSet;

/// What lies on disk below a folder of one library folder, the tool's own files left out: every
/// [`Self::folders`] (also empty ones) and every [`Self::files`], as paths from the library folder's root.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FolderEntries {
    pub folders: IndexSet<RelativePath>,
    pub files: IndexSet<RelativePath>,
}
