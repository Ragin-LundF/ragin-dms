use crate::DomainError;
use crate::catalog::{Catalog, CatalogUpdate};
use crate::error::require;

impl Catalog {
    /// The content of `snapshot`, an older copy of this library's catalog, as this catalog's next revision, so
    /// every folder and device takes it over.
    ///
    /// # Errors
    /// Fails when `snapshot` belongs to another library.
    pub fn restored_from(&self, snapshot: &Catalog) -> Result<CatalogUpdate, DomainError> {
        require(snapshot.library_id == self.library_id, || "The snapshot belongs to another library".to_owned())?;
        Ok(CatalogUpdate::Applied(Catalog { revision: self.revision.next(), ..snapshot.clone() }))
    }
}
