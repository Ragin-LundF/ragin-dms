use std::sync::Arc;

use dms_domain::cases::{CaseId, CaseStatus};

use crate::cases::case_commit::edit_case;
use crate::common::OperationOutcome;
use crate::library::CatalogStore;
use crate::port::system::Clock;

/// "Waiting for answer", "Mark as done" and "Reopen": the case's status from today.
pub struct SetCaseStatus {
    store: Arc<CatalogStore>,
    clock: Arc<dyn Clock>,
}

impl SetCaseStatus {
    pub fn new(store: Arc<CatalogStore>, clock: Arc<dyn Clock>) -> Self {
        Self { store, clock }
    }

    pub fn run(&self, case_id: &CaseId, status: CaseStatus) -> OperationOutcome {
        let today = self.clock.today();
        edit_case(&self.store, case_id, |_, case| Some(case.with_status(status, today)))
    }
}
