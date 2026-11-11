use dms_domain::catalog::Catalog;

use crate::port::storage::StorageError;

/// The result of reading the catalog file of one library folder.
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // read once per folder and unpacked right away
pub enum CatalogLoad {
    /// The catalog, plus any conflict copies a sync client created next to it.
    Found {
        catalog: Catalog,
        conflict_copies: Vec<Catalog>,
    },
    /// The folder has no catalog file yet.
    NotFound,
    Failed(StorageError),
}
