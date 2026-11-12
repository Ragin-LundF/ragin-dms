use std::sync::Arc;

use dms_domain::cases::{CaseId, CaseNote};
use dms_domain::text;

use crate::cases::case_commit::edit_case;
use crate::common::OperationOutcome;
use crate::library::CatalogStore;
use crate::port::system::Clock;

/// Adds a note dated today to a case; a blank note is refused.
pub struct AddCaseNote {
    store: Arc<CatalogStore>,
    clock: Arc<dyn Clock>,
}

impl AddCaseNote {
    pub fn new(store: Arc<CatalogStore>, clock: Arc<dyn Clock>) -> Self {
        Self { store, clock }
    }

    pub fn run(&self, case_id: &CaseId, note: &str) -> OperationOutcome {
        let note = CaseNote { date: self.clock.today(), text: text::trim(note).to_owned() };
        edit_case(&self.store, case_id, |_, case| Some(case.with_note(note.clone())))
    }
}
