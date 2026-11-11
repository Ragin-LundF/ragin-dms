use std::sync::Arc;

use dms_domain::document::{Document, DocumentId, DocumentKind};

use crate::common::LibrarySnapshot;
use crate::library::CatalogStore;
use crate::pdf::rebuild::RebuildBlocker;
use crate::port::pdf::PdfEditor;

/// Whether "Convert to real PDF" is offered for a document: a PDF without a real text layer. Whether the AI can read
/// images shows only when it is asked, so the first run says so if it cannot.
pub struct RebuildAvailability {
    store: Arc<CatalogStore>,
    pdf: Arc<dyn PdfEditor>,
}

impl RebuildAvailability {
    pub fn new(store: Arc<CatalogStore>, pdf: Arc<dyn PdfEditor>) -> Self {
        Self { store, pdf }
    }

    pub fn run(&self, document_id: &DocumentId) -> bool {
        let Some(library) = self.store.current() else {
            return false;
        };
        library
            .catalog
            .documents()
            .get(document_id)
            .is_some_and(|document| self.blocker_of(&library, document).is_none())
    }

    /// Why `document` cannot be rebuilt, or `None` if it can. A file that cannot be read is not offered.
    pub fn blocker_of(&self, library: &LibrarySnapshot, document: &Document) -> Option<RebuildBlocker> {
        if document.kind() != DocumentKind::Pdf {
            return Some(RebuildBlocker::NotPdf);
        }
        let has_text = self.pdf.has_text_layer(library.folders.primary(), &document.path).unwrap_or(true);
        has_text.then_some(RebuildBlocker::HasTextLayer)
    }
}
