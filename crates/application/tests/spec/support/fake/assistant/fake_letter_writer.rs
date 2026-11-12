use dms_application::port::assistant::{LetterAnswer, LetterDraft, LetterRequest, LetterWriter};

use crate::support::Shared;

/// Answers with `answer` and records the requests it was given.
pub struct FakeLetterWriter {
    pub answer: Shared<LetterAnswer>,
    pub requests: Shared<Vec<LetterRequest>>,
}

impl FakeLetterWriter {
    pub const MODEL: &'static str = "letter-model";
}

impl Default for FakeLetterWriter {
    fn default() -> Self {
        let draft = LetterDraft { subject: "AI subject".to_owned(), body: "AI body".to_owned() };
        Self {
            answer: Shared::new(LetterAnswer::Drafted { draft, model: Self::MODEL.to_owned() }),
            requests: Shared::default(),
        }
    }
}

impl LetterWriter for FakeLetterWriter {
    fn draft(&self, request: &LetterRequest) -> LetterAnswer {
        self.requests.update(|requests| requests.push(request.clone()));
        self.answer.get()
    }
}
