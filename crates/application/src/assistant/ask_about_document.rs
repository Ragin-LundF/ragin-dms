use std::sync::{Arc, Mutex, PoisonError};

use dms_domain::document::{Document, DocumentId};

use crate::assistant::{AnswerOutcome, AssistantOutcomes, DocumentReading, ReadingResult};
use crate::library::CatalogStore;
use crate::port::ai::DocumentContent;
use crate::port::assistant::{AssistantRequest, ChatTurn, DocumentAssistant};

/// Answers the user's questions about one document. The conversation so far goes with every question, so
/// follow-up questions work. The document is read once and kept while it is unchanged, because rendering pages for
/// every question would be slow.
pub struct AskAboutDocument {
    store: Arc<CatalogStore>,
    reading: Arc<DocumentReading>,
    assistant: Arc<dyn DocumentAssistant>,
    cached: Mutex<Option<(Document, DocumentContent)>>,
}

impl AskAboutDocument {
    pub fn new(store: Arc<CatalogStore>, reading: Arc<DocumentReading>, assistant: Arc<dyn DocumentAssistant>) -> Self {
        Self { store, reading, assistant, cached: Mutex::new(None) }
    }

    pub fn run(&self, document_id: &DocumentId, conversation: &[ChatTurn]) -> AnswerOutcome {
        let (document, content) = match self.cached_read(document_id) {
            Some(read) => read,
            None => match self.reading.read(document_id) {
                ReadingResult::Read { document, content, .. } => {
                    *self.cached.lock().unwrap_or_else(PoisonError::into_inner) =
                        Some((document.clone(), content.clone()));
                    (document, content)
                }
                problem => return AssistantOutcomes::of_problem(problem),
            },
        };
        let request = AssistantRequest { file_name: document.path.file_name().to_owned(), content };
        AssistantOutcomes::of_answer(self.assistant.answer(&request, conversation))
    }

    /// The cached document, while the catalog still describes it the same way.
    fn cached_read(&self, document_id: &DocumentId) -> Option<(Document, DocumentContent)> {
        let current = self.store.current().and_then(|library| library.catalog.documents().get(document_id).cloned());
        let cached = self.cached.lock().unwrap_or_else(PoisonError::into_inner);
        cached
            .as_ref()
            .filter(|(document, _)| &document.id == document_id && Some(document) == current.as_ref())
            .cloned()
    }
}
