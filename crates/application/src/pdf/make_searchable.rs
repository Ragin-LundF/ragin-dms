use std::sync::Arc;

use dms_domain::document::{Document, DocumentId, DocumentKind};
use dms_domain::text;

use crate::common::{ActivityRecorder, LibrarySnapshot, OperationOutcome};
use crate::library::CatalogStore;
use crate::pdf::{PdfVersions, SearchableOutcome};
use crate::port::pdf::{OcrTool, PdfEditor};
use crate::port::state::ActivityKind;
use crate::port::storage::TextStore;

/// Makes a scanned PDF searchable: a new version with an invisible text layer from the transcription the AI made of
/// it (one text per page), or, with `precise`, from the OCR program installed on this device. PDFs that already have
/// a text layer are left as they are.
pub struct MakeSearchable {
    store: Arc<CatalogStore>,
    pdf: Arc<dyn PdfEditor>,
    ocr: Arc<dyn OcrTool>,
    texts: Arc<dyn TextStore>,
    versions: Arc<PdfVersions>,
    activity: Arc<ActivityRecorder>,
}

/// The searchable version, or why there is none.
enum VersionResult {
    Written(OperationOutcome),
    Missing(SearchableOutcome),
}

impl MakeSearchable {
    /// Separates the pages of a transcription in the text store.
    pub const PAGE_BREAK: char = '\u{000C}';

    pub fn new(
        store: Arc<CatalogStore>,
        pdf: Arc<dyn PdfEditor>,
        ocr: Arc<dyn OcrTool>,
        texts: Arc<dyn TextStore>,
        versions: Arc<PdfVersions>,
        activity: Arc<ActivityRecorder>,
    ) -> Self {
        Self { store, pdf, ocr, texts, versions, activity }
    }

    pub fn run(&self, document_id: &DocumentId, precise: bool) -> SearchableOutcome {
        let Some(library) = self.store.current() else {
            return SearchableOutcome::Failed(OperationOutcome::LibraryNotOpen);
        };
        let Some(document) = library.catalog.documents().get(document_id) else {
            return SearchableOutcome::Failed(OperationOutcome::NotFound);
        };
        if document.kind() != DocumentKind::Pdf {
            return SearchableOutcome::NotPdf;
        }
        match self.pdf.has_text_layer(library.folders.primary(), &document.path) {
            Err(error) => return SearchableOutcome::Failed(OperationOutcome::StorageFailed(error)),
            Ok(true) => return SearchableOutcome::AlreadySearchable,
            Ok(false) => {}
        }
        let outcome = match self.new_version(&library, document, precise) {
            VersionResult::Missing(outcome) => return outcome,
            VersionResult::Written(outcome) => outcome,
        };
        if !matches!(outcome, OperationOutcome::Done { .. }) {
            return SearchableOutcome::Failed(outcome);
        }
        self.activity.record(ActivityKind::PdfEdited, &format!("Searchable: {}", document.path));
        SearchableOutcome::Made(outcome)
    }

    fn new_version(&self, library: &LibrarySnapshot, document: &Document, precise: bool) -> VersionResult {
        if precise {
            if !self.ocr.available() {
                return VersionResult::Missing(SearchableOutcome::OcrUnavailable);
            }
            return VersionResult::Written(
                self.versions.replace(library, document, |folder, target| self.ocr.ocr(folder, &document.path, target)),
            );
        }
        let transcription =
            self.texts.read(library.folders.primary(), &document.id).filter(|text| !text::is_blank(text));
        let Some(transcription) = transcription else {
            return VersionResult::Missing(SearchableOutcome::NoTranscription);
        };
        let pages: Vec<String> = transcription.split(Self::PAGE_BREAK).map(str::to_owned).collect();
        VersionResult::Written(self.versions.replace(library, document, |folder, target| {
            self.pdf.add_text_layer(folder, &document.path, &pages, target)
        }))
    }
}
