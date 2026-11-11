use dms_domain::document::DocumentId;
use dms_domain::query::DocumentQuery;

/// Which documents an export takes.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // passed by reference once per export
pub enum ExportScope {
    Documents(Vec<DocumentId>),
    /// The documents of a smart folder, a preset such as a tax year, or any other query.
    Query(DocumentQuery),
}
