use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use dms_domain::document::{ContentHash, Document, DocumentId};
use dms_domain::library::LibraryFolder;
use dms_domain::text;

use crate::port::ai::DocumentContentReader;
use crate::port::storage::TextStore;

/// The text of documents as written, for the index and the answers: the AI's transcription from the text store, or
/// the file's text layer. Read once per file version and kept in memory.
pub struct DocumentTexts {
    reader: Arc<dyn DocumentContentReader>,
    texts: Arc<dyn TextStore>,
    cache: Mutex<HashMap<DocumentId, (ContentHash, String)>>,
}

impl DocumentTexts {
    pub fn new(reader: Arc<dyn DocumentContentReader>, texts: Arc<dyn TextStore>) -> Self {
        Self { reader, texts, cache: Mutex::new(HashMap::new()) }
    }

    pub fn of(&self, folder: &LibraryFolder, document: &Document) -> String {
        let mut cache = self.cache.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some((hash, text)) = cache.get(&document.id)
            && *hash == document.content_hash
        {
            return text.clone();
        }
        let stored = self.texts.read(folder, &document.id).filter(|stored| !text::is_blank(stored));
        let text = stored.or_else(|| self.reader.text(folder, document)).unwrap_or_default();
        let cleaned = text.replace('\u{000C}', "\n");
        cache.insert(document.id.clone(), (document.content_hash.clone(), cleaned.clone()));
        cleaned
    }
}
