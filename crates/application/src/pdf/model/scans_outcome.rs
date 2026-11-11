use dms_domain::document::DocumentId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScansOutcome {
    /// The PDFs without a text layer, by path.
    Found(Vec<DocumentId>),
    LibraryNotOpen,
}
