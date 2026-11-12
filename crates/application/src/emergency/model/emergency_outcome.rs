use dms_domain::document::DocumentId;

use crate::common::OperationOutcome;

/// What became of saving the overview or writing the encrypted ZIP.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // returned once per user action
pub enum EmergencyOutcome {
    /// Written to `location` (a library path or an absolute path); `document_id` when it was catalogued, `documents`
    /// is the number of library documents it holds.
    Saved {
        location: String,
        document_id: Option<DocumentId>,
        documents: usize,
    },
    /// The chosen place lies inside a library folder.
    InsideLibrary,
    /// No item of the folder is included.
    NothingIncluded,
    Refused(OperationOutcome),
}
