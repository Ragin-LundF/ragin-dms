use chrono::NaiveDate;
use dms_domain::DomainError;
use dms_domain::catalog::{Catalog, CatalogUpdate};
use dms_domain::tax::{TaxChecklistBuilder, TaxItemId};

use crate::port::system::IdGenerator;

/// The catalog with `year`'s checklist built afresh; the catalog itself, unchanged, when the stored list is current.
pub(super) fn with_checklist(
    catalog: &Catalog,
    year: i32,
    today: NaiveDate,
    ids: &dyn IdGenerator,
) -> Result<CatalogUpdate, DomainError> {
    let built = TaxChecklistBuilder::build(catalog, year, today, &mut || {
        TaxItemId::new(ids.new_id()).expect("invariant: generated ids are not blank")
    });
    if catalog.tax_years().get(&year) == Some(&built) {
        return Ok(CatalogUpdate::Applied(catalog.clone()));
    }
    catalog.save_tax_year(built)
}

/// Applies `next` to the catalog of an applied update; a rejection or error stands.
pub(super) fn then(
    update: Result<CatalogUpdate, DomainError>,
    next: impl FnOnce(&Catalog) -> Result<CatalogUpdate, DomainError>,
) -> Result<CatalogUpdate, DomainError> {
    match update? {
        CatalogUpdate::Applied(catalog) => next(&catalog),
        rejected @ CatalogUpdate::Rejected(_) => Ok(rejected),
    }
}
