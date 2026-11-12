use dms_domain::document::DocumentId;

use crate::port::assistant::{LetterKind, LetterParty};

/// What a feature asks a letter for: its `kind`, the source documents it answers or refers to (may be empty), the
/// recipient's name if the feature knows it (otherwise the correspondent of the newest source), the facts the
/// feature knows, and the sender (the owner details of the library, if set).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftLetterInput {
    pub kind: LetterKind,
    pub document_ids: Vec<DocumentId>,
    pub recipient: Option<String>,
    pub subject_facts: Vec<String>,
    pub sender: Option<LetterParty>,
}
