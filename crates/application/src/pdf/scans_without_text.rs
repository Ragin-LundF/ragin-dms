use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use dms_domain::document::{ContentHash, DocumentKind};
use dms_domain::text;

use crate::library::CatalogStore;
use crate::pdf::ScansOutcome;
use crate::port::pdf::PdfEditor;

/// The PDFs without a text layer, such as scans without OCR. Each file is checked once and the answer kept until it
/// changes. Only reads.
pub struct ScansWithoutText {
    store: Arc<CatalogStore>,
    pdf: Arc<dyn PdfEditor>,
    known: Mutex<HashMap<ContentHash, bool>>,
}

impl ScansWithoutText {
    pub fn new(store: Arc<CatalogStore>, pdf: Arc<dyn PdfEditor>) -> Self {
        Self { store, pdf, known: Mutex::new(HashMap::new()) }
    }

    pub fn run(&self) -> ScansOutcome {
        let Some(library) = self.store.current() else {
            return ScansOutcome::LibraryNotOpen;
        };
        let mut known = self.known.lock().unwrap_or_else(PoisonError::into_inner);
        let mut scans: Vec<_> = library
            .catalog
            .documents()
            .values()
            .filter(|document| document.kind() == DocumentKind::Pdf)
            .filter(|document| {
                let has_text = *known.entry(document.content_hash.clone()).or_insert_with(|| {
                    // A file that cannot be read is not offered; making it searchable would fail too.
                    self.pdf.has_text_layer(library.folders.primary(), &document.path).unwrap_or(true)
                });
                !has_text
            })
            .collect();
        scans.sort_by(|first, second| text::compare(first.path.value(), second.path.value()));
        ScansOutcome::Found(scans.into_iter().map(|document| document.id.clone()).collect())
    }
}
