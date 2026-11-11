use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use dms_domain::document::{Document, DocumentId};
use dms_domain::library::LibraryFolder;
use dms_domain::query::{DocumentQuery, QueryContext};
use dms_domain::text;
use indexmap::IndexSet;

use crate::common::LibrarySnapshot;
use crate::port::ai::DocumentContentReader;
use crate::port::storage::TextStore;
use crate::port::system::Clock;
use crate::search::IndexedText;

/// Matches documents against their text, ignoring case. A document's text is what the AI transcribed of it, kept
/// in the text store, plus the text layer of its file, plus the user's note. File texts are read once and kept in
/// memory until the file changes, so only the first search reads every file. Shared by every use case that
/// searches, so they share that memory. Never writes.
pub struct ContentMatcher {
    reader: Arc<dyn DocumentContentReader>,
    texts: Arc<dyn TextStore>,
    clock: Arc<dyn Clock>,
    index: Mutex<HashMap<DocumentId, IndexedText>>,
}

impl ContentMatcher {
    pub fn new(reader: Arc<dyn DocumentContentReader>, texts: Arc<dyn TextStore>, clock: Arc<dyn Clock>) -> Self {
        Self { reader, texts, clock, index: Mutex::new(HashMap::new()) }
    }

    /// The documents whose text contains `wanted`; nothing for a blank `wanted`.
    pub fn containing(&self, library: &LibrarySnapshot, wanted: &str) -> IndexSet<DocumentId> {
        let normalized = text::lowercase(text::trim(wanted));
        if normalized.is_empty() {
            return IndexSet::new();
        }
        let mut index = self.index.lock().unwrap_or_else(PoisonError::into_inner);
        library
            .catalog
            .documents()
            .values()
            .filter(|document| self.contains_text(&mut index, library.folders.primary(), document, &normalized))
            .map(|document| document.id.clone())
            .collect()
    }

    /// The documents `query` matches, by path. Its content term is looked up in the documents' texts, only for the
    /// documents the other conditions leave.
    pub fn matching(&self, library: &LibrarySnapshot, query: &DocumentQuery) -> Vec<Document> {
        let context = QueryContext { catalog: &library.catalog, today: self.clock.today(), zone: self.clock.zone() };
        let candidates = query.filter(&context);
        let wanted = query.content_text.as_deref().map(|wanted| text::lowercase(text::trim(wanted)));
        let Some(wanted) = wanted.filter(|wanted| !wanted.is_empty()) else {
            return candidates.into_iter().cloned().collect();
        };
        let mut index = self.index.lock().unwrap_or_else(PoisonError::into_inner);
        candidates
            .into_iter()
            .filter(|document| self.contains_text(&mut index, library.folders.primary(), document, &wanted))
            .cloned()
            .collect()
    }

    fn contains_text(
        &self,
        index: &mut HashMap<DocumentId, IndexedText>,
        folder: &LibraryFolder,
        document: &Document,
        wanted: &str,
    ) -> bool {
        let note = document.note.as_deref().map(text::lowercase).unwrap_or_default();
        note.contains(wanted) || self.file_text(index, folder, document).contains(wanted)
    }

    fn file_text<'a>(
        &self,
        index: &'a mut HashMap<DocumentId, IndexedText>,
        folder: &LibraryFolder,
        document: &Document,
    ) -> &'a str {
        let known = index.get(&document.id).is_some_and(|known| known.content_hash == document.content_hash);
        if !known {
            let stored = self.texts.read(folder, &document.id);
            let extracted = self.reader.text(folder, document);
            let joined = [stored, extracted].into_iter().flatten().collect::<Vec<_>>().join("\n");
            let indexed = IndexedText { content_hash: document.content_hash.clone(), text: text::lowercase(&joined) };
            index.insert(document.id.clone(), indexed);
        }
        &index.get(&document.id).expect("invariant: indexed just now").text
    }
}
