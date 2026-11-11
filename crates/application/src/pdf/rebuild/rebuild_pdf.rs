use std::sync::Arc;

use dms_domain::document::{Document, DocumentId};
use dms_domain::rebuild::{KeepReason, LayoutLine, PageLayout};

use crate::common::{LibrarySnapshot, OperationOutcome, Progress};
use crate::library::CatalogStore;
use crate::pdf::rebuild::{LineRepair, RebuildAvailability, RebuildBlocker, RebuildOutcome};
use crate::port::ai::{ClassificationFailure, LayoutOutcome, LineOutcome, PageLayoutReader};
use crate::port::pdf::{PageInput, PdfEditor, PdfRebuilder, PreparedPage};
use crate::port::storage::StorageError;

/// "Convert to real PDF": rebuilds a scanned PDF page by page with the AI the user configured. The result waits
/// outside the library for the user's choice (`KeepRebuild`); nothing in the library changes here. A page the AI
/// cannot read in time stays as it was; an AI that cannot be reached or cannot read images stops the run.
/// `progress` hears `(page, pages)` before each page starts, and with `page == pages` before the file is written. A
/// PDF that changed during the run is not written, since its pages would come from two versions. A cancelled run
/// stops before the next page or before writing and leaves nothing behind, since the file is only written at the end.
pub struct RebuildPdf {
    store: Arc<CatalogStore>,
    pdf: Arc<dyn PdfEditor>,
    rebuilder: Arc<dyn PdfRebuilder>,
    reader: Arc<dyn PageLayoutReader>,
    availability: Arc<RebuildAvailability>,
}

impl RebuildPdf {
    pub fn new(
        store: Arc<CatalogStore>,
        pdf: Arc<dyn PdfEditor>,
        rebuilder: Arc<dyn PdfRebuilder>,
        reader: Arc<dyn PageLayoutReader>,
        availability: Arc<RebuildAvailability>,
    ) -> Self {
        Self { store, pdf, rebuilder, reader, availability }
    }

    pub fn run(&self, document_id: &DocumentId, progress: &dyn Progress<(usize, usize)>) -> RebuildOutcome {
        let Some(library) = self.store.current() else {
            return RebuildOutcome::Failed(OperationOutcome::LibraryNotOpen);
        };
        let Some(document) = library.catalog.documents().get(document_id) else {
            return RebuildOutcome::Failed(OperationOutcome::NotFound);
        };
        if let Some(blocker) = self.availability.blocker_of(&library, document) {
            return RebuildOutcome::NotAvailable(blocker);
        }
        let pages = match self.pdf.page_count(library.folders.primary(), &document.path) {
            Ok(pages) => pages,
            Err(error) => return failed(error),
        };
        let mut inputs = Vec::with_capacity(pages);
        for page in 0..pages {
            if progress.is_cancelled() {
                return RebuildOutcome::Cancelled;
            }
            progress.report(&(page, pages));
            match self.read_page(&library, document, page) {
                Ok(input) => inputs.push(input),
                Err(outcome) => return outcome,
            }
        }
        if progress.is_cancelled() {
            return RebuildOutcome::Cancelled;
        }
        progress.report(&(pages, pages));
        self.write(document, &inputs)
    }

    /// Writes the rebuild unless the document changed during the run.
    fn write(&self, document: &Document, inputs: &[PageInput]) -> RebuildOutcome {
        let Some(library) = self.store.current() else {
            return RebuildOutcome::Failed(OperationOutcome::LibraryNotOpen);
        };
        let Some(current) = library.catalog.documents().get(&document.id) else {
            return RebuildOutcome::Failed(OperationOutcome::NotFound);
        };
        if current.content_hash != document.content_hash {
            return RebuildOutcome::NotAvailable(RebuildBlocker::SourceChanged);
        }
        match self.rebuilder.write(library.folders.primary(), &current.path, &document.content_hash, inputs) {
            Err(error) => failed(error),
            Ok(rebuilt) => RebuildOutcome::Rebuilt { rebuilt, original_size: document.size },
        }
    }

    #[allow(clippy::result_large_err)] // the refusal is the outcome of the whole action, built once
    fn read_page(
        &self,
        library: &LibrarySnapshot,
        document: &Document,
        page: usize,
    ) -> Result<PageInput, RebuildOutcome> {
        let prepared = self.rebuilder.prepare_page(library.folders.primary(), &document.path, page).map_err(failed)?;
        match self.reader.read_layout(&prepared.image) {
            LayoutOutcome::Read(layout) => {
                let layout = self.repaired(layout, &prepared);
                Ok(PageInput::Read { prepared, layout })
            }
            LayoutOutcome::Failure { reason, .. } => match blocker_of(reason) {
                Some(blocker) => Err(RebuildOutcome::NotAvailable(blocker)),
                None => Ok(PageInput::Failed { page_index: page }),
            },
        }
    }

    /// Lines with lost characters, read again from a crop; a line that does not fit stays an image.
    fn repaired(&self, layout: PageLayout, prepared: &PreparedPage) -> PageLayout {
        let lines = layout
            .lines
            .into_iter()
            .map(
                |line| {
                    if LineRepair::has_lost_characters(&line.text) { self.repair_line(line, prepared) } else { line }
                },
            )
            .collect();
        PageLayout { lines, ..layout }
    }

    fn repair_line(&self, line: LayoutLine, prepared: &PreparedPage) -> LayoutLine {
        let crop = self.rebuilder.crop(prepared, &line.bounds);
        let text = match self.reader.read_line(&crop) {
            LineOutcome::Read(reread) => LineRepair::repaired(&line.text, &reread),
            LineOutcome::Failure { .. } => None,
        };
        match text {
            Some(text) => LayoutLine { text, ..line },
            None => LayoutLine { kept: Some(KeepReason::LostCharacters), ..line },
        }
    }
}

/// An AI that cannot be reached or cannot read images will not read the next page either.
fn blocker_of(reason: ClassificationFailure) -> Option<RebuildBlocker> {
    match reason {
        ClassificationFailure::ProviderUnavailable => Some(RebuildBlocker::AiNotReachable),
        ClassificationFailure::ModelCannotReadImages => Some(RebuildBlocker::ModelCannotReadImages),
        ClassificationFailure::Timeout | ClassificationFailure::InvalidResponse => None,
    }
}

#[allow(clippy::needless_pass_by_value)] // used as `map_err(failed)`
fn failed(error: StorageError) -> RebuildOutcome {
    RebuildOutcome::Failed(OperationOutcome::StorageFailed(error))
}
