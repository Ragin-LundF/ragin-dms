//! The text of every page of a stack: the text layer, else the stored transcription if it kept the page boundaries,
//! else the vision model page by page.

use dms_domain::document::Document;
use dms_domain::library::LibraryFolder;
use dms_domain::text;

use crate::common::Progress;
use crate::pdf::MakeSearchable;
use crate::port::ai::{ClassificationFailure, LineOutcome};
use crate::port::assistant::SplitProposer;
use crate::port::pdf::PdfEditor;
use crate::port::storage::{StorageError, TextStore};
use crate::split::{PageSource, SplitProgress};

/// The long side of a page picture for the vision model, as the classifier sends it.
const VISION_SIDE: u32 = 1600;

pub(super) enum PageTexts {
    Read { texts: Vec<String>, source: PageSource },
    Cancelled,
    Unreadable(StorageError),
    Failed { reason: ClassificationFailure, detail: Option<String> },
}

pub(super) struct StackPages<'a> {
    pub pdf: &'a dyn PdfEditor,
    pub texts: &'a dyn TextStore,
    pub proposer: &'a dyn SplitProposer,
}

impl StackPages<'_> {
    pub(super) fn read(
        &self,
        folder: &LibraryFolder,
        document: &Document,
        blank: &[bool],
        progress: &dyn Progress<SplitProgress>,
    ) -> PageTexts {
        match self.pdf.has_text_layer(folder, &document.path) {
            Err(error) => return PageTexts::Unreadable(error),
            Ok(true) => {
                return match self.pdf.page_texts(folder, &document.path) {
                    Ok(texts) => PageTexts::Read { texts, source: PageSource::TextLayer },
                    Err(error) => PageTexts::Unreadable(error),
                };
            }
            Ok(false) => {}
        }
        if let Some(texts) = self.transcribed_pages(folder, document, blank.len()) {
            return PageTexts::Read { texts, source: PageSource::Transcription };
        }
        self.read_by_vision(folder, document, blank, progress)
    }

    fn transcribed_pages(&self, folder: &LibraryFolder, document: &Document, page_count: usize) -> Option<Vec<String>> {
        let transcription = self.texts.read(folder, &document.id).filter(|text| !text::is_blank(text))?;
        let pages: Vec<String> = transcription.split(MakeSearchable::PAGE_BREAK).map(str::to_owned).collect();
        (pages.len() == page_count).then_some(pages)
    }

    /// Blank pages are not sent; a page that cannot be rendered counts as one without text.
    fn read_by_vision(
        &self,
        folder: &LibraryFolder,
        document: &Document,
        blank: &[bool],
        progress: &dyn Progress<SplitProgress>,
    ) -> PageTexts {
        let total = blank.len();
        let mut texts = Vec::with_capacity(total);
        progress.report(&SplitProgress { done: 0, total });
        for (page, is_blank) in blank.iter().enumerate() {
            if progress.is_cancelled() {
                return PageTexts::Cancelled;
            }
            let picture =
                (!is_blank).then(|| self.pdf.render_page(folder, &document.path, page, VISION_SIDE)).flatten();
            let text = match picture.map(|picture| self.proposer.read_page(&picture)) {
                None => String::new(),
                Some(LineOutcome::Read(text)) => text,
                Some(LineOutcome::Failure { reason, detail }) => return PageTexts::Failed { reason, detail },
            };
            texts.push(text);
            progress.report(&SplitProgress { done: page + 1, total });
        }
        PageTexts::Read { texts, source: PageSource::Vision }
    }
}
