use dms_application::port::assistant::{AssistantAnswer, AssistantRequest, ChatTurn, DocumentAssistant};
use dms_domain::taxonomy::TagName;
use indexmap::IndexSet;

use crate::support::Shared;
use crate::support::test_data::tag;

/// Answers with `summary`, `tags` and `answer` and records what it was asked.
pub struct FakeAssistant {
    pub summary: Shared<AssistantAnswer>,
    pub tags: Shared<AssistantAnswer>,
    pub answer: Shared<AssistantAnswer>,
    pub requests: Shared<Vec<AssistantRequest>>,
    pub conversations: Shared<Vec<Vec<ChatTurn>>>,
}

impl FakeAssistant {
    pub const MODEL: &'static str = "test-model";
}

impl Default for FakeAssistant {
    fn default() -> Self {
        Self {
            summary: Shared::new(AssistantAnswer::Text {
                text: "A synthetic summary".into(),
                model: Self::MODEL.into(),
            }),
            tags: Shared::new(AssistantAnswer::Tags { tags: vec![tag("Sample")], model: Self::MODEL.into() }),
            answer: Shared::new(AssistantAnswer::Text { text: "A synthetic answer".into(), model: Self::MODEL.into() }),
            requests: Shared::default(),
            conversations: Shared::default(),
        }
    }
}

impl DocumentAssistant for FakeAssistant {
    fn summarize(&self, request: &AssistantRequest) -> AssistantAnswer {
        self.requests.update(|requests| requests.push(request.clone()));
        self.summary.get()
    }

    fn suggest_tags(&self, request: &AssistantRequest, _existing_tags: &IndexSet<TagName>) -> AssistantAnswer {
        self.requests.update(|requests| requests.push(request.clone()));
        self.tags.get()
    }

    fn answer(&self, request: &AssistantRequest, conversation: &[ChatTurn]) -> AssistantAnswer {
        self.requests.update(|requests| requests.push(request.clone()));
        self.conversations.update(|conversations| conversations.push(conversation.to_vec()));
        self.answer.get()
    }
}
