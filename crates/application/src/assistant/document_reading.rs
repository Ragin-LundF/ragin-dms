use std::sync::Arc;

use dms_domain::document::DocumentId;

use crate::assistant::ReadingResult;
use crate::library::CatalogStore;
use crate::port::ai::{DocumentContent, DocumentContentReader};

/// Reads a document of the open library from the primary folder, for the AI. Never changes it.
pub struct DocumentReading {
    store: Arc<CatalogStore>,
    reader: Arc<dyn DocumentContentReader>,
}

impl DocumentReading {
    pub fn new(store: Arc<CatalogStore>, reader: Arc<dyn DocumentContentReader>) -> Self {
        Self { store, reader }
    }

    pub fn read(&self, document_id: &DocumentId) -> ReadingResult {
        let Some(library) = self.store.current() else {
            return ReadingResult::LibraryNotOpen;
        };
        let Some(document) = library.catalog.documents().get(document_id).cloned() else {
            return ReadingResult::NotFound;
        };
        if !document.kind().readable_by_ai() {
            return ReadingResult::NotReadable { detail: None };
        }
        match self.reader.read(library.folders.primary(), &document) {
            DocumentContent::NotReadable(reason) => ReadingResult::NotReadable { detail: Some(reason) },
            content => ReadingResult::Read { document, catalog: library.catalog, content },
        }
    }
}
