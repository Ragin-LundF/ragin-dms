use std::ops::Deref;
use std::sync::Arc;

use chrono::NaiveDate;
use dms_application::letter::{DraftLetter, SaveLetter};
use dms_domain::document::{Document, DocumentMetadata};

use crate::support::TestLibrary;
use crate::support::fake::assistant::FakeLetterWriter;
use crate::support::fake::pdf::FakePdfEditor;
use crate::support::test_data::done;

/// A `TestLibrary` with the PDF editor and the letter writer fakes.
pub struct LetterKit {
    pub library: TestLibrary,
    pub pdf: Arc<FakePdfEditor>,
    pub writer: Arc<FakeLetterWriter>,
}

impl Deref for LetterKit {
    type Target = TestLibrary;

    fn deref(&self) -> &TestLibrary {
        &self.library
    }
}

impl LetterKit {
    pub fn new() -> Self {
        let library = TestLibrary::new();
        library.open();
        Self {
            pdf: Arc::new(FakePdfEditor::new(library.folders.clone())),
            writer: Arc::new(FakeLetterWriter::default()),
            library,
        }
    }

    pub fn draft_letter(&self) -> DraftLetter {
        DraftLetter::new(
            self.store.clone(),
            self.reader.clone(),
            self.folders.clone(),
            self.settings.clone(),
            self.writer.clone(),
        )
    }

    pub fn save_letter(&self) -> SaveLetter {
        SaveLetter::new(
            self.store.clone(),
            self.pdf.clone(),
            self.replicator.clone(),
            self.names.clone(),
            self.activity.clone(),
            self.clock.clone(),
            self.ids.clone(),
        )
    }

    pub fn enable_ai(&self) {
        self.settings.preferences.update(|preferences| preferences.ai.enabled = true);
    }

    /// Adds a document from `correspondent`, dated `date`, whose text layer is `text`.
    pub fn add_from(&self, raw_path: &str, correspondent: &str, date: NaiveDate, text: &str) -> Document {
        let document = self.add(raw_path, raw_path);
        self.reader.set_text(document.path.file_name(), text);
        let metadata =
            DocumentMetadata { title: None, document_date: Some(date), correspondent: Some(correspondent.to_owned()) };
        done(self.store.commit(|catalog| catalog.update_metadata(&document.id, metadata.clone())));
        self.catalog().documents().get(&document.id).cloned().expect("still catalogued")
    }
}
