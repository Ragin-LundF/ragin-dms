use std::collections::BTreeSet;
use std::sync::Arc;

use dms_domain::DomainError;
use dms_domain::catalog::{Catalog, CatalogRejection, CatalogUpdate};
use dms_domain::document::DocumentId;
use dms_domain::tax::TaxAssignment;

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;
use crate::port::system::{Clock, IdGenerator};
use crate::tax::checklist_refresh::{then, with_checklist};

/// Assigns documents to a tax year and area, or with `None` removes their assignment, in one save. With
/// `keep_deductible` every document keeps its own deductible part (the bulk bar); otherwise the given one is set
/// (the quick look). The checklists of the touched years that exist are refreshed in the same save, so an item is
/// ticked as soon as its document is assigned.
pub struct AssignTax {
    store: Arc<CatalogStore>,
    activity: Arc<ActivityRecorder>,
    clock: Arc<dyn Clock>,
    ids: Arc<dyn IdGenerator>,
}

impl AssignTax {
    pub fn new(
        store: Arc<CatalogStore>,
        activity: Arc<ActivityRecorder>,
        clock: Arc<dyn Clock>,
        ids: Arc<dyn IdGenerator>,
    ) -> Self {
        Self { store, activity, clock, ids }
    }

    pub fn run(
        &self,
        document_ids: &[DocumentId],
        tax: Option<&TaxAssignment>,
        keep_deductible: bool,
    ) -> OperationOutcome {
        let today = self.clock.today();
        let outcome = self.store.commit(|catalog| {
            if document_ids.iter().any(|id| !catalog.documents().contains_key(id)) {
                return Ok(CatalogUpdate::Rejected(CatalogRejection::DocumentNotFound));
            }
            let years: BTreeSet<i32> = document_ids
                .iter()
                .filter_map(|id| catalog.documents().get(id)?.tax.as_ref().map(|tax| tax.year))
                .chain(tax.map(|tax| tax.year))
                .filter(|year| catalog.tax_years().contains_key(year))
                .collect();
            let assigned = document_ids.iter().try_fold(CatalogUpdate::Applied(catalog.clone()), |update, id| {
                then(Ok(update), |current| assign(current, id, tax, keep_deductible))
            });
            years.into_iter().fold(assigned, |update, year| {
                then(update, |current| with_checklist(current, year, today, self.ids.as_ref()))
            })
        });
        if matches!(outcome, OperationOutcome::Done { .. }) {
            self.activity
                .record(ActivityKind::CatalogEdited, &format!("Tax assignment of {} documents", document_ids.len()));
        }
        outcome
    }
}

fn assign(
    catalog: &Catalog,
    id: &DocumentId,
    tax: Option<&TaxAssignment>,
    keep_deductible: bool,
) -> Result<CatalogUpdate, DomainError> {
    let current = catalog.documents().get(id).and_then(|document| document.tax.as_ref());
    let wanted = tax.map(|tax| TaxAssignment {
        deductible: if keep_deductible {
            current.and_then(|known| known.deductible.clone())
        } else {
            tax.deductible.clone()
        },
        ..tax.clone()
    });
    if current == wanted.as_ref() {
        return Ok(CatalogUpdate::Applied(catalog.clone()));
    }
    catalog.set_tax(id, wanted)
}
