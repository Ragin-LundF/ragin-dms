use std::sync::Arc;

use chrono::NaiveDate;
use dms_domain::document::DocumentId;
use indexmap::IndexMap;

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;

/// Marks a bill as paid on a day: its due date task disappears and it leaves the open bills.
pub struct MarkBillPaid {
    store: Arc<CatalogStore>,
    activity: Arc<ActivityRecorder>,
}

impl MarkBillPaid {
    pub fn new(store: Arc<CatalogStore>, activity: Arc<ActivityRecorder>) -> Self {
        Self { store, activity }
    }

    pub fn run(&self, document_id: &DocumentId, paid_on: NaiveDate) -> OperationOutcome {
        let outcome = self
            .store
            .commit(|catalog| Ok(catalog.set_paid_on(&IndexMap::from([(document_id.clone(), Some(paid_on))]))));
        if let OperationOutcome::Done { catalog, .. } = &outcome
            && let Some(document) = catalog.documents().get(document_id)
        {
            self.activity.record(ActivityKind::CatalogEdited, &format!("Paid {}", document.path));
        }
        outcome
    }
}
