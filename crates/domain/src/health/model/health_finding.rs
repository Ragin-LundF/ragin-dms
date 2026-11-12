use crate::document::DocumentId;
use crate::library::LibraryFolderId;

/// A problem the archive check found that a scan cannot see.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HealthFinding {
    /// The copy in `folder_id` differs from the catalog while the copy in `source_folder_id` still matches it.
    Damaged { document_id: DocumentId, folder_id: LibraryFolderId, source_folder_id: LibraryFolderId },
    /// The PDF opens in no folder that has it.
    Unreadable { document_id: DocumentId },
}

impl HealthFinding {
    #[must_use]
    pub fn document_id(&self) -> &DocumentId {
        match self {
            Self::Damaged { document_id, .. } | Self::Unreadable { document_id } => document_id,
        }
    }
}
