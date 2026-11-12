use crate::document::ContentHash;
use crate::library::LibraryFolderId;

/// What a check found out about the copy of one document in one library folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyObservation {
    pub folder_id: LibraryFolderId,
    pub state: CopyState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CopyState {
    /// The folder has no file at the document's path; the scan's sync tasks deal with it.
    Missing,
    /// A sync client keeps the file only in the cloud; the check never downloads it.
    NotDownloaded,
    /// The file is there but reading it failed, as with a damaged disk.
    ReadFailed,
    /// The file was read. `opens` is `None` for documents that are no PDF.
    Read { content_hash: ContentHash, size: u64, opens: Option<bool> },
}
