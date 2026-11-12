use dms_domain::document::DocumentId;
use dms_domain::library::{LibraryFolderId, RelativePath};
use indexmap::IndexSet;

use crate::common::OperationOutcome;

/// The saved letter's document and file, with the sync folders that get it when retried; or why nothing (or only
/// the file, which the next scan finds) was saved.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // returned once per saved letter
pub enum SaveLetterOutcome {
    Saved { document_id: DocumentId, path: RelativePath, pending_folders: IndexSet<LibraryFolderId> },
    Refused(OperationOutcome),
}
