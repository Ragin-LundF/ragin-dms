use std::sync::Arc;

use dms_domain::cases::CaseId;

use crate::common::OperationOutcome;
use crate::library::CatalogStore;

/// Deletes a case after the user confirmed it; its documents stay in the library.
pub struct DeleteCase {
    store: Arc<CatalogStore>,
}

impl DeleteCase {
    pub fn new(store: Arc<CatalogStore>) -> Self {
        Self { store }
    }

    pub fn run(&self, case_id: &CaseId, confirmed: bool) -> OperationOutcome {
        if !confirmed {
            return OperationOutcome::NotConfirmed;
        }
        self.store.commit(|catalog| Ok(catalog.delete_case(case_id)))
    }
}
