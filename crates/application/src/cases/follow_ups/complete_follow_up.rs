use std::sync::Arc;

use dms_domain::cases::{CaseId, FollowUpId};

use crate::cases::case_commit::edit_case;
use crate::common::OperationOutcome;
use crate::library::CatalogStore;

/// Marks a follow-up as done; it no longer appears on the dashboard.
pub struct CompleteFollowUp {
    store: Arc<CatalogStore>,
}

impl CompleteFollowUp {
    pub fn new(store: Arc<CatalogStore>) -> Self {
        Self { store }
    }

    pub fn run(&self, case_id: &CaseId, follow_up: &FollowUpId) -> OperationOutcome {
        edit_case(&self.store, case_id, |_, case| case.completing(follow_up))
    }
}
