use std::sync::Arc;

use dms_domain::DomainError;
use dms_domain::catalog::{Catalog, CatalogRejection, CatalogUpdate};
use dms_domain::tax::{TaxAssignment, TaxChecklistItem, TaxItemId, TaxItemState, TaxYear};
use dms_domain::text;

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;
use crate::port::system::{Clock, IdGenerator};
use crate::tax::TaxItemChange;
use crate::tax::checklist_refresh::{then, with_checklist};

/// Changes one item of a tax year's checklist: adds the user's own, renames, marks "not needed this year" or links
/// the document the user has for it ("I have it", which also assigns that document to the year and area). The
/// checklist is refreshed in the same save. An unknown item is [`OperationOutcome::NotFound`].
pub struct UpdateTaxItem {
    store: Arc<CatalogStore>,
    activity: Arc<ActivityRecorder>,
    clock: Arc<dyn Clock>,
    ids: Arc<dyn IdGenerator>,
}

impl UpdateTaxItem {
    pub fn new(
        store: Arc<CatalogStore>,
        activity: Arc<ActivityRecorder>,
        clock: Arc<dyn Clock>,
        ids: Arc<dyn IdGenerator>,
    ) -> Self {
        Self { store, activity, clock, ids }
    }

    pub fn run(&self, year: i32, change: &TaxItemChange) -> OperationOutcome {
        let today = self.clock.today();
        let ids = self.ids.as_ref();
        let outcome = self.store.commit(|catalog| {
            let opened = with_checklist(catalog, year, today, ids);
            let changed = then(opened, |current| {
                let tax_year = current.tax_years().get(&year).cloned().unwrap_or_else(|| TaxYear::new(year));
                match changed_year(current, tax_year, change, ids)? {
                    Some((tax_year, catalog)) => catalog.save_tax_year(tax_year),
                    None => Ok(CatalogUpdate::Rejected(CatalogRejection::DocumentNotFound)),
                }
            });
            then(changed, |current| with_checklist(current, year, today, ids))
        });
        // An unknown item or document, or one removed meanwhile.
        if outcome == OperationOutcome::Rejected(CatalogRejection::DocumentNotFound) {
            return OperationOutcome::NotFound;
        }
        if matches!(outcome, OperationOutcome::Done { .. }) {
            self.activity.record(ActivityKind::CatalogEdited, &format!("Tax year {year}"));
        }
        outcome
    }
}

/// The changed checklist and the catalog to save it in; `None` for an unknown item or document.
fn changed_year(
    catalog: &Catalog,
    mut tax_year: TaxYear,
    change: &TaxItemChange,
    ids: &dyn IdGenerator,
) -> Result<Option<(TaxYear, Catalog)>, DomainError> {
    let index_of = |tax_year: &TaxYear, id: &TaxItemId| tax_year.items.iter().position(|item| &item.id == id);
    match change {
        TaxItemChange::Add { area, label, correspondent } => {
            tax_year.items.push(TaxChecklistItem {
                id: TaxItemId::new(ids.new_id()).expect("invariant: generated ids are not blank"),
                area: *area,
                label: text::trim(label).to_owned(),
                correspondent: correspondent
                    .as_deref()
                    .map(text::trim)
                    .filter(|name| !name.is_empty())
                    .map(str::to_owned),
                document_id: None,
                state: TaxItemState::Missing,
                note: None,
                added_by_user: true,
            });
        }
        TaxItemChange::Rename { item_id, label } => {
            let Some(index) = index_of(&tax_year, item_id) else { return Ok(None) };
            text::trim(label).clone_into(&mut tax_year.items[index].label);
        }
        TaxItemChange::SetNotNeeded { item_id, not_needed } => {
            let Some(index) = index_of(&tax_year, item_id) else { return Ok(None) };
            tax_year.items[index].state = if *not_needed { TaxItemState::NotNeeded } else { TaxItemState::Missing };
        }
        TaxItemChange::LinkDocument { item_id, document_id } => {
            let (Some(index), Some(document)) = (index_of(&tax_year, item_id), catalog.documents().get(document_id))
            else {
                return Ok(None);
            };
            let item = &mut tax_year.items[index];
            item.document_id = Some(document_id.clone());
            item.state = TaxItemState::Found;
            let deductible = document.tax.as_ref().and_then(|tax| tax.deductible.clone());
            let assignment = TaxAssignment { year: tax_year.year, area: item.area, deductible };
            return match catalog.set_tax(document_id, Some(assignment))? {
                CatalogUpdate::Applied(assigned) => Ok(Some((tax_year, assigned))),
                CatalogUpdate::Rejected(_) => Ok(None),
            };
        }
    }
    Ok(Some((tax_year, catalog.clone())))
}
