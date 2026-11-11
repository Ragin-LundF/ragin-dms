use dms_domain::document::DocumentId;

/// A passage found for a question, with how well it fits: the higher `score`, the better. `start` and `end` are
/// UTF-16 offsets into the trimmed text, as in Kotlin.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoredChunk {
    pub document_id: DocumentId,
    pub start: usize,
    pub end: usize,
    pub score: f64,
}
