use crate::document::DocumentId;
use crate::library::RelativePath;
use crate::restructure::SkipReason;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SkippedDocument {
    pub document_id: DocumentId,
    pub path: RelativePath,
    pub reason: SkipReason,
}
