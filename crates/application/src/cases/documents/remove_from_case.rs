use std::sync::Arc;

use dms_domain::cases::CaseId;
use dms_domain::document::DocumentId;

use crate::cases::case_commit::edit_case;
use crate::common::OperationOutcome;
use crate::library::CatalogStore;

/// Takes a document out of a case; the document stays in the library.
pub struct RemoveFromCase {
    store: Arc<CatalogStore>,
}

impl RemoveFromCase {
    pub fn new(store: Arc<CatalogStore>) -> Self {
        Self { store }
    }

    pub fn run(&self, case_id: &CaseId, document: &DocumentId) -> OperationOutcome {
        edit_case(&self.store, case_id, |_, case| Some(case.without_document(document)))
    }
}
