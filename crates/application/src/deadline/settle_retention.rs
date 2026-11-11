use std::sync::Arc;

use dms_domain::catalog::CatalogUpdate;
use dms_domain::document::DocumentId;
use dms_domain::retention::Retention;

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;
use crate::port::system::Clock;

/// Marks the ended retention of documents as done: they stay ordinary documents and are never flagged again, also
/// not through their category. One catalog change for all of them.
pub struct SettleRetention {
    store: Arc<CatalogStore>,
    activity: Arc<ActivityRecorder>,
    clock: Arc<dyn Clock>,
}

impl SettleRetention {
    pub fn new(store: Arc<CatalogStore>, activity: Arc<ActivityRecorder>, clock: Arc<dyn Clock>) -> Self {
        Self { store, activity, clock }
    }

    pub fn run(&self, document_ids: &[DocumentId]) -> OperationOutcome {
        let now = self.clock.now();
        let outcome = self.store.commit(|catalog| {
            let start = CatalogUpdate::Applied(catalog.clone());
            Ok(document_ids.iter().fold(start, |update, document_id| {
                update.and_then(|current| {
                    let kept = current
                        .documents()
                        .get(document_id)
                        .and_then(|document| document.retention.clone())
                        .unwrap_or_default();
                    current.set_retention(document_id, Some(Retention { settled_at: Some(now), ..kept }))
                })
            }))
        });
        if matches!(outcome, OperationOutcome::Done { .. }) {
            self.activity
                .record(ActivityKind::CatalogEdited, &format!("Retention done for {} documents", document_ids.len()));
        }
        outcome
    }
}
