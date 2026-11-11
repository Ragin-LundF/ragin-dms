use indexmap::{IndexMap, IndexSet};

use crate::document::{ContentHash, DocumentId};
use crate::library::{LibraryFolderId, RelativePath};

/// A difference between the catalog and the library folders, found by [`crate::change::ChangeDetector`].
///
/// Each variant holds its own struct, like the Kotlin subtypes, so a task can hold exactly one kind of change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectedChange {
    NewFile(NewFile),
    ModifiedFile(ModifiedFile),
    MovedFile(MovedFile),
    MissingFile(MissingFile),
    ContentConflict(ContentConflict),
}

/// A file the catalog does not know, present in the listed folders with the same content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewFile {
    pub path: RelativePath,
    pub present_in: IndexSet<LibraryFolderId>,
    pub size: u64,
    pub content_hash: Option<ContentHash>,
}

/// A catalogued file whose content differs from the catalog in one folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModifiedFile {
    pub document_id: DocumentId,
    pub path: RelativePath,
    pub folder_id: LibraryFolderId,
    pub size: u64,
    pub content_hash: Option<ContentHash>,
}

/// A catalogued file found under another path in one folder, recognized by its content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovedFile {
    pub document_id: DocumentId,
    pub path: RelativePath,
    pub previous_path: RelativePath,
    pub folder_id: LibraryFolderId,
}

/// A catalogued file missing from some folders. [`Self::present_in`] lists the folders that still have it; when
/// it is empty, the file is gone from every reachable folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingFile {
    pub document_id: DocumentId,
    pub path: RelativePath,
    pub missing_in: IndexSet<LibraryFolderId>,
    pub present_in: IndexSet<LibraryFolderId>,
}

/// A file the catalog does not know, with different content in different folders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentConflict {
    pub path: RelativePath,
    pub content_hashes: IndexMap<LibraryFolderId, ContentHash>,
}

impl DetectedChange {
    #[must_use]
    pub fn path(&self) -> &RelativePath {
        match self {
            Self::NewFile(change) => &change.path,
            Self::ModifiedFile(change) => &change.path,
            Self::MovedFile(change) => &change.path,
            Self::MissingFile(change) => &change.path,
            Self::ContentConflict(change) => &change.path,
        }
    }
}
