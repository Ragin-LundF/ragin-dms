use crate::document::DocumentId;

/// Who proposed a follow-up: the user, or the AI from a letter that asked for something. The document is only the
/// origin; it may have left the library since.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FollowUpSource {
    Manual,
    Ai { document_id: DocumentId },
}
