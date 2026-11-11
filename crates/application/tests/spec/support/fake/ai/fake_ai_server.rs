use dms_application::port::ai::{AiServer, ModelList};
use dms_application::port::state::AiSettings;

use crate::support::Shared;

/// Answers every request with `answer` and records the settings it was asked with.
pub struct FakeAiServer {
    pub answer: Shared<ModelList>,
    pub requests: Shared<Vec<AiSettings>>,
}

impl Default for FakeAiServer {
    fn default() -> Self {
        Self { answer: Shared::new(ModelList::Available(Vec::new())), requests: Shared::default() }
    }
}

impl AiServer for FakeAiServer {
    fn models(&self, settings: &AiSettings) -> ModelList {
        self.requests.update(|requests| requests.push(settings.clone()));
        self.answer.get()
    }
}
