use std::sync::Arc;

use dms_domain::DomainError;
use dms_domain::catalog::{Catalog, CatalogUpdate};
use dms_domain::link::DocumentLink;

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::document::DetailEdit;
use crate::library::CatalogStore;
use crate::port::state::ActivityKind;

/// Applies one change to a document's fields, note, retention or links, as one catalog change.
pub struct EditDocumentDetails {
    store: Arc<CatalogStore>,
    activity: Arc<ActivityRecorder>,
}

impl EditDocumentDetails {
    pub fn new(store: Arc<CatalogStore>, activity: Arc<ActivityRecorder>) -> Self {
        Self { store, activity }
    }

    pub fn run(&self, edit: &DetailEdit) -> OperationOutcome {
        let outcome = self.store.commit(|catalog| apply(catalog, edit));
        if matches!(outcome, OperationOutcome::Done { .. }) {
            self.activity.record(ActivityKind::CatalogEdited, edit.name());
        }
        outcome
    }
}

fn apply(catalog: &Catalog, edit: &DetailEdit) -> Result<CatalogUpdate, DomainError> {
    match edit {
        DetailEdit::SetFields { document_id, fields } => catalog.set_fields(document_id, fields.clone()),
        DetailEdit::SetNote { document_id, note } => Ok(catalog.set_note(document_id, note.as_deref())),
        DetailEdit::SetRetention { document_id, retention } => {
            Ok(catalog.set_retention(document_id, retention.clone()))
        }
        DetailEdit::SetRetentionYears { category_id, years } => catalog.set_retention_years(category_id, *years),
        DetailEdit::Link { first, second, label } => {
            Ok(catalog.link(DocumentLink::new(first.clone(), second.clone(), label.as_deref())?))
        }
        DetailEdit::Unlink { first, second } => Ok(catalog.unlink(first, second)),
    }
}
