use std::sync::Arc;

use dms_domain::document::DocumentId;

use crate::assistant::{AnswerOutcome, AssistantOutcomes, DocumentReading, ReadingResult};
use crate::port::assistant::{AssistantRequest, DocumentAssistant};

/// Lets the AI summarize one document. Only shows the summary; nothing is stored.
pub struct SummarizeDocument {
    reading: Arc<DocumentReading>,
    assistant: Arc<dyn DocumentAssistant>,
}

impl SummarizeDocument {
    pub fn new(reading: Arc<DocumentReading>, assistant: Arc<dyn DocumentAssistant>) -> Self {
        Self { reading, assistant }
    }

    pub fn run(&self, document_id: &DocumentId) -> AnswerOutcome {
        let (document, content) = match self.reading.read(document_id) {
            ReadingResult::Read { document, content, .. } => (document, content),
            problem => return AssistantOutcomes::of_problem(problem),
        };
        let request = AssistantRequest { file_name: document.path.file_name().to_owned(), content };
        AssistantOutcomes::of_answer(self.assistant.summarize(&request))
    }
}
