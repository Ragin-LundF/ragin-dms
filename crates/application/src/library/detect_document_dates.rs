use std::cell::Cell;
use std::sync::Arc;

use chrono::NaiveDate;
use dms_domain::catalog::{CatalogRejection, CatalogUpdate};
use dms_domain::document::{DocumentId, DocumentMetadata};
use indexmap::IndexMap;

use crate::common::{ActivityRecorder, DocumentDates, OperationOutcome};
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;

/// Gives every document without a date the one [`DocumentDates`] finds, for documents catalogued before dates were
/// detected. Dates the user or an approved suggestion set are never touched.
pub struct DetectDocumentDates {
    store: Arc<CatalogStore>,
    dates: Arc<DocumentDates>,
    activity: Arc<ActivityRecorder>,
}

impl DetectDocumentDates {
    pub fn new(store: Arc<CatalogStore>, dates: Arc<DocumentDates>, activity: Arc<ActivityRecorder>) -> Self {
        Self { store, dates, activity }
    }

    /// Returns how many documents got a date.
    pub fn run(&self) -> usize {
        let Some(library) = self.store.current() else {
            return 0;
        };
        let found: IndexMap<DocumentId, NaiveDate> = library
            .catalog
            .documents()
            .values()
            .filter(|document| document.metadata.document_date.is_none())
            .filter_map(|document| {
                self.dates.guess(library.folders.primary(), &document.path).map(|date| (document.id.clone(), date))
            })
            .collect();
        if found.is_empty() {
            return 0;
        }
        let dated = Cell::new(0);
        let outcome = self.store.commit(|catalog| {
            let mut current = catalog.clone();
            dated.set(0);
            for (id, date) in &found {
                let Some(document) = current.documents().get(id) else { continue };
                if document.metadata.document_date.is_some() {
                    continue;
                }
                let metadata = DocumentMetadata { document_date: Some(*date), ..document.metadata.clone() };
                if let CatalogUpdate::Applied(next) = current.update_metadata(id, metadata)? {
                    current = next;
                    dated.set(dated.get() + 1);
                }
            }
            Ok(if dated.get() == 0 {
                CatalogUpdate::Rejected(CatalogRejection::DocumentNotFound)
            } else {
                CatalogUpdate::Applied(current)
            })
        });
        if !matches!(outcome, OperationOutcome::Done { .. }) {
            return 0;
        }
        self.activity.record(ActivityKind::CatalogEdited, &format!("Dates of {} documents", dated.get()));
        dated.get()
    }
}
