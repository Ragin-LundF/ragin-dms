use std::sync::Arc;

use dms_domain::document::{DocumentId, DocumentKind};

use crate::classification::{EInvoiceSuggestions, SuggestOutcome};
use crate::library::CatalogStore;
use crate::port::ai::DocumentContentReader;
use crate::port::state::TaskStore;

/// Suggests the exact values of the e-invoice embedded in a PDF, without the AI, for the user's review.
/// `NotReadable` when the document holds no e-invoice.
pub struct SuggestFromEInvoice {
    store: Arc<CatalogStore>,
    reader: Arc<dyn DocumentContentReader>,
    tasks: Arc<dyn TaskStore>,
}

impl SuggestFromEInvoice {
    pub fn new(store: Arc<CatalogStore>, reader: Arc<dyn DocumentContentReader>, tasks: Arc<dyn TaskStore>) -> Self {
        Self { store, reader, tasks }
    }

    pub fn run(&self, document_id: &DocumentId) -> SuggestOutcome {
        let Some(library) = self.store.current() else {
            return SuggestOutcome::LibraryNotOpen;
        };
        let Some(document) = library.catalog.documents().get(document_id) else {
            return SuggestOutcome::NotFound;
        };
        let e_invoice = (document.kind() == DocumentKind::Pdf)
            .then(|| self.reader.read_e_invoice(library.folders.primary(), document))
            .flatten();
        let Some(e_invoice) = e_invoice else {
            return SuggestOutcome::NotReadable { detail: None };
        };
        let suggestion = EInvoiceSuggestions::of(document, &e_invoice, &library.catalog);
        // Like SuggestClassification: the suggestion is shown even when the task store cannot be written.
        let _ = self.tasks.save_suggestion(&suggestion);
        SuggestOutcome::Suggested(suggestion)
    }
}
