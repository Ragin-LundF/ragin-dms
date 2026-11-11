use crate::port::ai::{ClassificationRequest, ClassificationResult};

/// Suggests title, category, tags, date, correspondent and file name. Suggestions only, never applied.
pub trait DocumentClassifier: Send + Sync {
    fn suggest(&self, request: &ClassificationRequest) -> ClassificationResult;
}
