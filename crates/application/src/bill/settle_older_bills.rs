use std::sync::Arc;

use dms_domain::bill::BillPlanner;
use dms_domain::catalog::CatalogUpdate;
use indexmap::IndexMap;

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;

/// "Mark as settled" for the older bills never marked: each gets its due date as the day it was paid, in one
/// change. Needs the user's confirmation.
pub struct SettleOlderBills {
    store: Arc<CatalogStore>,
    activity: Arc<ActivityRecorder>,
}

impl SettleOlderBills {
    pub fn new(store: Arc<CatalogStore>, activity: Arc<ActivityRecorder>) -> Self {
        Self { store, activity }
    }

    pub fn run(&self, confirmed: bool) -> OperationOutcome {
        if !confirmed {
            return OperationOutcome::NotConfirmed;
        }
        let settled = std::cell::Cell::new(0);
        let outcome = self.store.commit(|catalog| {
            let older: IndexMap<_, _> =
                BillPlanner::never_marked(catalog).into_iter().map(|(id, due)| (id, Some(due))).collect();
            settled.set(older.len());
            Ok(if older.is_empty() { CatalogUpdate::Applied(catalog.clone()) } else { catalog.set_paid_on(&older) })
        });
        if matches!(outcome, OperationOutcome::Done { .. }) && settled.get() > 0 {
            self.activity.record(ActivityKind::CatalogEdited, &format!("Settled {} older bills", settled.get()));
        }
        outcome
    }
}
