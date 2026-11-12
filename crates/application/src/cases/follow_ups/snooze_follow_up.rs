use std::sync::Arc;

use dms_domain::cases::{CaseId, FollowUpId};

use crate::cases::case_commit::edit_case;
use crate::common::OperationOutcome;
use crate::library::CatalogStore;
use crate::port::system::Clock;

/// Moves a follow-up `days` later (from today when its date has passed), so the dashboard asks again then.
pub struct SnoozeFollowUp {
    store: Arc<CatalogStore>,
    clock: Arc<dyn Clock>,
}

impl SnoozeFollowUp {
    pub fn new(store: Arc<CatalogStore>, clock: Arc<dyn Clock>) -> Self {
        Self { store, clock }
    }

    pub fn run(&self, case_id: &CaseId, follow_up: &FollowUpId, days: u16) -> OperationOutcome {
        let today = self.clock.today();
        edit_case(&self.store, case_id, |_, case| case.snoozing(follow_up, days, today))
    }
}
