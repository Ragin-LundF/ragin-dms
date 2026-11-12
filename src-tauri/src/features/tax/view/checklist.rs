//! The checklist a screen shows for a year: the stored one, or a preview built from the library while it was never
//! stored (its items then carry preview ids and offer no actions).

use chrono::NaiveDate;
use dms_domain::catalog::Catalog;
use dms_domain::tax::{TaxChecklistBuilder, TaxItemId, TaxItemState, TaxYear};

pub struct Checklist {
    pub year: TaxYear,
    pub stored: bool,
}

impl Checklist {
    pub fn of(catalog: &Catalog, year: i32, today: NaiveDate) -> Self {
        if let Some(stored) = catalog.tax_years().get(&year) {
            return Self { year: stored.clone(), stored: true };
        }
        let mut next = 0;
        let preview = TaxChecklistBuilder::build(catalog, year, today, &mut || {
            next += 1;
            TaxItemId::new(format!("preview-{next}")).expect("invariant: not blank")
        });
        Self { year: preview, stored: false }
    }

    /// Found items, and every item except the ones not needed this year.
    pub fn progress(&self) -> (u32, u32) {
        let count = |wanted: &dyn Fn(TaxItemState) -> bool| {
            u32::try_from(self.year.items.iter().filter(|item| wanted(item.state)).count()).unwrap_or(u32::MAX)
        };
        (count(&|state| state == TaxItemState::Found), count(&|state| state != TaxItemState::NotNeeded))
    }
}
