use dms_application::port::ai::{ClassificationRequest, ClassificationResult, DocumentClassifier};
use dms_domain::classification::ClassificationSuggestion;

use crate::support::Shared;

/// Answers with `result`, or with an empty suggestion for the requested document. Records every request.
#[derive(Default)]
pub struct FakeClassifier {
    pub result: Shared<Option<ClassificationResult>>,
    pub requests: Shared<Vec<ClassificationRequest>>,
}

impl FakeClassifier {
    pub const MODEL: &'static str = "test-model";
}

impl DocumentClassifier for FakeClassifier {
    fn suggest(&self, request: &ClassificationRequest) -> ClassificationResult {
        self.requests.update(|requests| requests.push(request.clone()));
        self.result.get().unwrap_or_else(|| {
            ClassificationResult::Suggested(ClassificationSuggestion::empty(request.document_id.clone(), Self::MODEL))
        })
    }
}
