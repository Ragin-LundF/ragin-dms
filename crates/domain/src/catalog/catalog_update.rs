use crate::catalog::{Catalog, CatalogRejection};

/// The result of a change to a [`Catalog`].
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // moved once per change; boxing would only add noise for every caller
pub enum CatalogUpdate {
    /// The change was applied; the catalog carries the next revision.
    Applied(Catalog),
    /// The change was refused; the original catalog is unchanged.
    Rejected(CatalogRejection),
}

impl CatalogUpdate {
    /// Applies `next` to the resulting catalog; a rejection short-circuits and stays a rejection.
    #[must_use]
    pub fn and_then(self, next: impl FnOnce(Catalog) -> Self) -> Self {
        match self {
            Self::Applied(catalog) => next(catalog),
            Self::Rejected(_) => self,
        }
    }
}
