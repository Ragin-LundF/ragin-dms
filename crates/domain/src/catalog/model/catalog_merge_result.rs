use crate::catalog::{Catalog, CatalogMergeConflict};

/// The merged catalog plus every decision the merge had to make.
#[derive(Debug, Clone, PartialEq)]
pub struct CatalogMergeResult {
    pub catalog: Catalog,
    pub conflicts: Vec<CatalogMergeConflict>,
}
