use dms_domain::document::DocumentId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelatedOutcome {
    /// The closest documents, most similar first; none while the document is not in the index.
    Found(Vec<DocumentId>),
    LibraryNotOpen,
    NotFound,
}
