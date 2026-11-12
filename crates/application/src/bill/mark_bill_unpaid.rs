use std::sync::Arc;

use dms_domain::document::DocumentId;
use indexmap::IndexMap;

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;

/// Undoes "Mark as paid": the bill is open (or overdue) again and its due date task comes back.
pub struct MarkBillUnpaid {
    store: Arc<CatalogStore>,
    activity: Arc<ActivityRecorder>,
}

impl MarkBillUnpaid {
    pub fn new(store: Arc<CatalogStore>, activity: Arc<ActivityRecorder>) -> Self {
        Self { store, activity }
    }

    pub fn run(&self, document_id: &DocumentId) -> OperationOutcome {
        let outcome =
            self.store.commit(|catalog| Ok(catalog.set_paid_on(&IndexMap::from([(document_id.clone(), None)]))));
        if let OperationOutcome::Done { catalog, .. } = &outcome
            && let Some(document) = catalog.documents().get(document_id)
        {
            self.activity.record(ActivityKind::CatalogEdited, &format!("Not paid: {}", document.path));
        }
        outcome
    }
}
