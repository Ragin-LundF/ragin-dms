use dms_domain::catalog::Catalog;
use dms_domain::library::LibraryFolders;

/// The folders and the catalog of the open library at one moment.
#[derive(Debug, Clone, PartialEq)]
pub struct LibrarySnapshot {
    pub folders: LibraryFolders,
    pub catalog: Catalog,
}
