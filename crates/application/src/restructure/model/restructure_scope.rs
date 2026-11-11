use dms_domain::library::RelativePath;
use dms_domain::query::DocumentQuery;

/// Which documents a restructure looks at.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)] // one per planned restructure
pub enum RestructureScope {
    /// Every document directly or further below the path; the root means the whole library.
    Folder(RelativePath),
    /// The documents of a saved search or any other query.
    Query(DocumentQuery),
}
