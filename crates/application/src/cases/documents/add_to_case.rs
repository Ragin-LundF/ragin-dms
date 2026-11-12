use std::sync::Arc;

use dms_domain::cases::{CaseId, EntryDirection};
use dms_domain::document::DocumentId;

use crate::cases::case_commit::{edit_case, entry_date};
use crate::common::OperationOutcome;
use crate::library::CatalogStore;
use crate::port::system::Clock;

/// Puts a document into a case as received or sent, dated by its document date. A received document answers a
/// waiting case, which opens again.
pub struct AddToCase {
    store: Arc<CatalogStore>,
    clock: Arc<dyn Clock>,
}

impl AddToCase {
    pub fn new(store: Arc<CatalogStore>, clock: Arc<dyn Clock>) -> Self {
        Self { store, clock }
    }

    pub fn run(&self, case_id: &CaseId, document: &DocumentId, direction: EntryDirection) -> OperationOutcome {
        let today = self.clock.today();
        edit_case(&self.store, case_id, |catalog, case| {
            Some(case.with_entry(document, direction, entry_date(catalog, document, today)))
        })
    }
}
