use dms_application::port::ai::{LineOutcome, PageImage};
use dms_application::port::assistant::{SplitAnswer, SplitProposer, SplitRequest};

use crate::support::Shared;

/// Reads a page picture as its bytes in UTF-8, answers proposals with `answer`, and records both.
pub struct FakeSplitProposer {
    pub answer: Shared<SplitAnswer>,
    pub reading_fails: Shared<bool>,
    pub pages_read: Shared<Vec<String>>,
    pub requests: Shared<Vec<SplitRequest>>,
}

impl FakeSplitProposer {
    pub const MODEL: &'static str = "split-model";
}

impl Default for FakeSplitProposer {
    fn default() -> Self {
        Self {
            answer: Shared::new(SplitAnswer::Proposed { parts: Vec::new(), model: Self::MODEL.to_owned() }),
            reading_fails: Shared::default(),
            pages_read: Shared::default(),
            requests: Shared::default(),
        }
    }
}

impl SplitProposer for FakeSplitProposer {
    fn read_page(&self, page: &PageImage) -> LineOutcome {
        if self.reading_fails.get() {
            return LineOutcome::Failure {
                reason: dms_application::port::ai::ClassificationFailure::ModelCannotReadImages,
                detail: None,
            };
        }
        let text = String::from_utf8_lossy(&page.bytes).into_owned();
        self.pages_read.update(|pages| pages.push(text.clone()));
        LineOutcome::Read(format!("read: {text}"))
    }

    fn propose(&self, request: &SplitRequest) -> SplitAnswer {
        self.requests.update(|requests| requests.push(request.clone()));
        self.answer.get()
    }
}
