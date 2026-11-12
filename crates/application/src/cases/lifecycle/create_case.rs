use std::sync::Arc;

use dms_domain::cases::{Case, CaseId};
use dms_domain::text;

use crate::cases::case_commit::{entry_date, follow_up_of};
use crate::cases::{CreateCaseOutcome, NewCase};
use crate::library::CatalogStore;
use crate::port::system::{Clock, IdGenerator};

/// Opens a case today with its first documents (each dated by its document date) and follow-ups.
pub struct CreateCase {
    store: Arc<CatalogStore>,
    clock: Arc<dyn Clock>,
    ids: Arc<dyn IdGenerator>,
}

impl CreateCase {
    pub fn new(store: Arc<CatalogStore>, clock: Arc<dyn Clock>, ids: Arc<dyn IdGenerator>) -> Self {
        Self { store, clock, ids }
    }

    pub fn run(&self, new: &NewCase) -> CreateCaseOutcome {
        let case_id = CaseId::new(self.ids.new_id()).expect("invariant: generated ids are not blank");
        let today = self.clock.today();
        let follow_ups: Vec<_> = new.follow_ups.iter().map(|follow_up| follow_up_of(follow_up, &*self.ids)).collect();
        let correspondent =
            new.correspondent.as_deref().map(text::trim).filter(|name| !name.is_empty()).map(str::to_owned);
        let outcome = self.store.commit(|catalog| {
            let mut case = Case::new(case_id.clone(), text::trim(&new.title), today)?;
            case.correspondent.clone_from(&correspondent);
            for (document, direction) in &new.documents {
                case = case.with_entry(document, *direction, entry_date(catalog, document, today));
            }
            case.follow_ups.clone_from(&follow_ups);
            catalog.save_case(case)
        });
        CreateCaseOutcome { case_id, outcome }
    }
}
