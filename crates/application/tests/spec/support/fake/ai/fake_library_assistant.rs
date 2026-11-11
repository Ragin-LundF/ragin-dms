use dms_application::port::assistant::{AssistantAnswer, LibraryAssistant, SourceChunk};

use crate::support::Shared;

/// Answers with `answer` and records the passages it was given.
pub struct FakeLibraryAssistant {
    pub answer: Shared<String>,
    pub sources: Shared<Vec<Vec<SourceChunk>>>,
}

impl Default for FakeLibraryAssistant {
    fn default() -> Self {
        Self { answer: Shared::new("See [1].".to_owned()), sources: Shared::default() }
    }
}

impl LibraryAssistant for FakeLibraryAssistant {
    fn answer(&self, _question: &str, sources: &[SourceChunk]) -> AssistantAnswer {
        self.sources.update(|all| all.push(sources.to_vec()));
        AssistantAnswer::Text { text: self.answer.get(), model: "fake-chat".to_owned() }
    }
}
