use dms_domain::document::ContentHash;

/// The lower-case searchable `text` of a document, as it was for the file with `content_hash`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedText {
    pub content_hash: ContentHash,
    pub text: String,
}
