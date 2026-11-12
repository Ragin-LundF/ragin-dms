use std::sync::Arc;

use crate::common::OperationOutcome;
use crate::library::CatalogStore;
use crate::port::system::{Clock, IdGenerator};
use crate::tax::checklist_refresh::with_checklist;

/// Creates or refreshes a tax year's checklist when the tax screen shows it. Only the checklist changes, never a
/// document, and the catalog is written only when the list changed.
pub struct OpenTaxYear {
    store: Arc<CatalogStore>,
    clock: Arc<dyn Clock>,
    ids: Arc<dyn IdGenerator>,
}

impl OpenTaxYear {
    pub fn new(store: Arc<CatalogStore>, clock: Arc<dyn Clock>, ids: Arc<dyn IdGenerator>) -> Self {
        Self { store, clock, ids }
    }

    pub fn run(&self, year: i32) -> OperationOutcome {
        let today = self.clock.today();
        self.store.commit(|catalog| with_checklist(catalog, year, today, self.ids.as_ref()))
    }
}
