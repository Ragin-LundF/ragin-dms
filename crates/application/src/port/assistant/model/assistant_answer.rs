use dms_domain::taxonomy::TagName;

use crate::port::ai::ClassificationFailure;

/// What the AI answered, from `model`, or why it did not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssistantAnswer {
    Text { text: String, model: String },
    Tags { tags: Vec<TagName>, model: String },
    Failed { reason: ClassificationFailure, detail: Option<String> },
}
