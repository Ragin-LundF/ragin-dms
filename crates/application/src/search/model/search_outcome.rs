use dms_domain::document::DocumentId;
use indexmap::IndexSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchOutcome {
    /// The documents whose text contains the query; none for a blank query.
    Found(IndexSet<DocumentId>),
    LibraryNotOpen,
}
