use std::sync::Arc;

use dms_domain::document::{DocumentId, DocumentKind};
use dms_domain::split::SplitProposal;
use dms_domain::text;

use super::stack_pages::{PageTexts, StackPages};
use crate::common::Progress;
use crate::library::CatalogStore;
use crate::port::assistant::{SplitAnswer, SplitPage, SplitProposer, SplitRequest};
use crate::port::pdf::PdfEditor;
use crate::port::storage::TextStore;
use crate::split::{ProposeSplitOutcome, SplitProgress};

/// The AI's proposal where each document of a scanned PDF starts. Only reads; nothing changes until the user
/// applies the cuts.
pub struct ProposeSplit {
    store: Arc<CatalogStore>,
    pdf: Arc<dyn PdfEditor>,
    texts: Arc<dyn TextStore>,
    proposer: Arc<dyn SplitProposer>,
}

impl ProposeSplit {
    /// The characters of each page the AI gets; the start of a page tells where a document begins.
    pub const PAGE_TEXT_LIMIT: usize = 800;

    pub fn new(
        store: Arc<CatalogStore>,
        pdf: Arc<dyn PdfEditor>,
        texts: Arc<dyn TextStore>,
        proposer: Arc<dyn SplitProposer>,
    ) -> Self {
        Self { store, pdf, texts, proposer }
    }

    /// Reading the pages with the vision model reports each page and stops when `progress` is cancelled.
    pub fn run(&self, document_id: &DocumentId, progress: &dyn Progress<SplitProgress>) -> ProposeSplitOutcome {
        let Some(library) = self.store.current() else {
            return ProposeSplitOutcome::LibraryNotOpen;
        };
        let Some(document) = library.catalog.documents().get(document_id) else {
            return ProposeSplitOutcome::NotFound;
        };
        if document.kind() != DocumentKind::Pdf {
            return ProposeSplitOutcome::NotPdf;
        }
        let folder = library.folders.primary();
        let blank = match self.pdf.blank_pages(folder, &document.path) {
            Ok(blank) => blank,
            Err(error) => return ProposeSplitOutcome::Unreadable(error),
        };
        let page_count = blank.len();
        if page_count < 2 {
            return ProposeSplitOutcome::TooFewPages;
        }
        let pages = StackPages { pdf: self.pdf.as_ref(), texts: self.texts.as_ref(), proposer: self.proposer.as_ref() };
        let (texts, source) = match pages.read(folder, document, &blank, progress) {
            PageTexts::Read { texts, source } => (texts, source),
            PageTexts::Cancelled => return ProposeSplitOutcome::Cancelled,
            PageTexts::Unreadable(error) => return ProposeSplitOutcome::Unreadable(error),
            PageTexts::Failed { reason, detail } => return ProposeSplitOutcome::Failed { reason, detail },
        };
        let request = SplitRequest {
            pages: texts
                .iter()
                .zip(&blank)
                .map(|(text, blank)| SplitPage { text: shortened(text), blank: *blank })
                .collect(),
            categories: library.catalog.categories().values().cloned().collect(),
        };
        match self.proposer.propose(&request) {
            SplitAnswer::Failed { reason, detail } => ProposeSplitOutcome::Failed { reason, detail },
            SplitAnswer::Proposed { parts, model } => ProposeSplitOutcome::Proposed {
                proposal: SplitProposal::checked(parts, page_count),
                model,
                page_count,
                source,
            },
        }
    }
}

fn shortened(text: &str) -> String {
    text::trim(text).chars().take(ProposeSplit::PAGE_TEXT_LIMIT).collect()
}
