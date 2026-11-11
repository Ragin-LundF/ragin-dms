use std::sync::Arc;

use dms_domain::classification::{ClassificationSuggestion, SuggestedTag};
use dms_domain::document::DocumentId;
use dms_domain::text;
use indexmap::IndexSet;

use crate::assistant::{DocumentReading, ReadingResult};
use crate::classification::SuggestOutcome;
use crate::port::ai::ClassificationFailure;
use crate::port::assistant::{AssistantAnswer, AssistantRequest, DocumentAssistant};
use crate::port::state::TaskStore;

const DETAIL_LENGTH: usize = 300;

/// Lets the AI suggest tags for one document. Tags the document already has are left out. A suggestion with tags
/// waits for the user's review like any other suggestion; one without new tags is returned but not stored.
pub struct SuggestTags {
    reading: Arc<DocumentReading>,
    assistant: Arc<dyn DocumentAssistant>,
    tasks: Arc<dyn TaskStore>,
}

impl SuggestTags {
    pub fn new(
        reading: Arc<DocumentReading>,
        assistant: Arc<dyn DocumentAssistant>,
        tasks: Arc<dyn TaskStore>,
    ) -> Self {
        Self { reading, assistant, tasks }
    }

    pub fn run(&self, document_id: &DocumentId) -> SuggestOutcome {
        let (document, catalog, content) = match self.reading.read(document_id) {
            ReadingResult::Read { document, catalog, content } => (document, catalog, content),
            ReadingResult::LibraryNotOpen => return SuggestOutcome::LibraryNotOpen,
            ReadingResult::NotFound => return SuggestOutcome::NotFound,
            ReadingResult::NotReadable { detail } => return SuggestOutcome::NotReadable { detail },
        };
        let request = AssistantRequest { file_name: document.path.file_name().to_owned(), content };
        match self.assistant.suggest_tags(&request, catalog.tags()) {
            AssistantAnswer::Failed { reason, detail } => SuggestOutcome::Failed { reason, detail },
            AssistantAnswer::Text { text: answer, .. } => SuggestOutcome::Failed {
                reason: ClassificationFailure::InvalidResponse,
                detail: Some(text::take_utf16(&answer, DETAIL_LENGTH).to_owned()),
            },
            AssistantAnswer::Tags { tags, model } => {
                let new_tags: IndexSet<_> = tags.into_iter().filter(|tag| !document.tags.contains(tag)).collect();
                let suggestion = ClassificationSuggestion {
                    tags: new_tags
                        .into_iter()
                        .map(|tag| SuggestedTag { is_new: !catalog.tags().contains(&tag), name: tag })
                        .collect(),
                    ..ClassificationSuggestion::empty(document_id.clone(), model)
                };
                if !suggestion.tags.is_empty() {
                    // Kotlin threw when the task store could not be written; the suggestion is shown either way.
                    let _ = self.tasks.save_suggestion(&suggestion);
                }
                SuggestOutcome::Suggested(suggestion)
            }
        }
    }
}
