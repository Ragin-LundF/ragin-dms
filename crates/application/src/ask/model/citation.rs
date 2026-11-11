use dms_domain::document::DocumentId;

/// A source the answer cites as `[number]`: the document it came from.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Citation {
    pub number: usize,
    pub document_id: DocumentId,
    pub title: String,
}
