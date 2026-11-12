use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use dms_application::port::ai::{DocumentContent, DocumentContentReader};
use dms_domain::bill::EInvoice;
use dms_domain::document::Document;
use dms_domain::library::LibraryFolder;

use crate::support::Shared;

/// Reads every document as `content`; `texts` are the text layers and `e_invoices` the embedded e-invoices by file
/// name.
pub struct FakeContentReader {
    pub content: Shared<DocumentContent>,
    pub texts: Shared<HashMap<String, String>>,
    pub e_invoices: Shared<HashMap<String, EInvoice>>,
    text_reads: AtomicUsize,
}

impl FakeContentReader {
    pub fn text_reads(&self) -> usize {
        self.text_reads.load(Ordering::SeqCst)
    }

    pub fn set_text(&self, file_name: &str, text: &str) {
        self.texts.update(|texts| texts.insert(file_name.to_owned(), text.to_owned()));
    }
}

impl Default for FakeContentReader {
    fn default() -> Self {
        Self {
            content: Shared::new(DocumentContent::Text("synthetic text".to_owned())),
            texts: Shared::default(),
            e_invoices: Shared::default(),
            text_reads: AtomicUsize::new(0),
        }
    }
}

impl DocumentContentReader for FakeContentReader {
    fn read(&self, _folder: &LibraryFolder, _document: &Document) -> DocumentContent {
        self.content.get()
    }

    fn text(&self, _folder: &LibraryFolder, document: &Document) -> Option<String> {
        self.text_reads.fetch_add(1, Ordering::SeqCst);
        self.texts.lock().get(document.path.file_name()).cloned()
    }

    fn read_e_invoice(&self, _folder: &LibraryFolder, document: &Document) -> Option<EInvoice> {
        self.e_invoices.lock().get(document.path.file_name()).cloned()
    }
}
