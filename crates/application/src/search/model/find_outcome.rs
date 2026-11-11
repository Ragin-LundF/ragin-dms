use dms_domain::document::Document;

#[derive(Debug, Clone, PartialEq)]
pub enum FindOutcome {
    /// The documents the query matches, by path.
    Found(Vec<Document>),
    LibraryNotOpen,
}
