use crate::document::AddedFile;

#[derive(Debug, Clone, PartialEq)]
pub enum AddDocumentsOutcome {
    /// One entry per file, in the given order, each with its own outcome.
    Added(Vec<AddedFile>),
    LibraryNotOpen,
}
