use std::sync::Arc;

use dms_domain::document::{DocumentId, DocumentKind};

use crate::common::{ActivityRecorder, OperationOutcome};
use crate::library::CatalogStore;
use crate::pdf::PdfVersions;
use crate::port::pdf::{PageChange, PdfEditor};
use crate::port::state::ActivityKind;

/// Rotates, deletes and reorders the pages of a PDF document in place: `pages` lists the pages of the new version
/// in their order. The old version goes to the system trash in every folder.
pub struct EditPages {
    store: Arc<CatalogStore>,
    pdf: Arc<dyn PdfEditor>,
    versions: Arc<PdfVersions>,
    activity: Arc<ActivityRecorder>,
}

impl EditPages {
    pub fn new(
        store: Arc<CatalogStore>,
        pdf: Arc<dyn PdfEditor>,
        versions: Arc<PdfVersions>,
        activity: Arc<ActivityRecorder>,
    ) -> Self {
        Self { store, pdf, versions, activity }
    }

    pub fn run(&self, document_id: &DocumentId, pages: &[PageChange]) -> OperationOutcome {
        let Some(library) = self.store.current() else {
            return OperationOutcome::LibraryNotOpen;
        };
        let Some(document) = library.catalog.documents().get(document_id) else {
            return OperationOutcome::NotFound;
        };
        if document.kind() != DocumentKind::Pdf || pages.is_empty() {
            return OperationOutcome::NotFound;
        }
        let outcome = self
            .versions
            .replace(&library, document, |folder, target| self.pdf.rewrite(folder, &document.path, pages, target));
        if matches!(outcome, OperationOutcome::Done { .. }) {
            self.activity.record(ActivityKind::PdfEdited, document.path.value());
        }
        outcome
    }
}
