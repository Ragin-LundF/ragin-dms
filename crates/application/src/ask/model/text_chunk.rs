/// A passage of a document's text: UTF-16 units `start` to `end` of the trimmed text (Kotlin string indices, which
/// are also what `.dms-index/` stores).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextChunk {
    pub start: usize,
    pub end: usize,
    pub text: String,
}
