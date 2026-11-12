use std::sync::Arc;

use dms_domain::cases::CaseId;

use crate::cases::NewFollowUp;
use crate::cases::case_commit::{edit_case, follow_up_of};
use crate::common::OperationOutcome;
use crate::library::CatalogStore;
use crate::port::system::IdGenerator;

/// Adds a follow-up to a case; the dashboard asks for it from its date on.
pub struct AddFollowUp {
    store: Arc<CatalogStore>,
    ids: Arc<dyn IdGenerator>,
}

impl AddFollowUp {
    pub fn new(store: Arc<CatalogStore>, ids: Arc<dyn IdGenerator>) -> Self {
        Self { store, ids }
    }

    pub fn run(&self, case_id: &CaseId, follow_up: &NewFollowUp) -> OperationOutcome {
        let follow_up = follow_up_of(follow_up, &*self.ids);
        edit_case(&self.store, case_id, |_, case| Some(case.with_follow_up(follow_up.clone())))
    }
}
