use std::sync::Arc;

use dms_domain::document::DocumentKind;
use dms_domain::text;

use crate::common::Progress;
use crate::library::CatalogStore;
use crate::pdf::{MakeSearchable, SearchableProgress};

/// Makes every PDF of the library searchable that has no text layer yet, one after the other; PDFs with text are
/// skipped. Cancelling stops it between two documents; pausing is stopping and starting again, since done documents
/// are skipped.
pub struct MakeAllSearchable {
    store: Arc<CatalogStore>,
    make_searchable: Arc<MakeSearchable>,
}

impl MakeAllSearchable {
    pub fn new(store: Arc<CatalogStore>, make_searchable: Arc<MakeSearchable>) -> Self {
        Self { store, make_searchable }
    }

    pub fn run(&self, precise: bool, progress: &dyn Progress<SearchableProgress>) -> Option<SearchableProgress> {
        let library = self.store.current()?;
        let mut pdfs: Vec<_> =
            library.catalog.documents().values().filter(|document| document.kind() == DocumentKind::Pdf).collect();
        pdfs.sort_by(|first, second| text::compare(first.path.value(), second.path.value()));
        let mut last = None;
        for (index, document) in pdfs.iter().enumerate() {
            if progress.is_cancelled() {
                break;
            }
            let outcome = self.make_searchable.run(&document.id, precise);
            let step =
                SearchableProgress { done: index + 1, total: pdfs.len(), document_id: document.id.clone(), outcome };
            progress.report(&step);
            last = Some(step);
        }
        last
    }
}
