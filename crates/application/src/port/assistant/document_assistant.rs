use dms_domain::taxonomy::TagName;
use indexmap::IndexSet;

use crate::port::assistant::{AssistantAnswer, AssistantRequest, ChatTurn};

/// Lets the AI read one document and answer about it: a summary, tags, or a question the user asked. Answers are
/// shown to the user; nothing is applied without the user's approval.
pub trait DocumentAssistant: Send + Sync {
    fn summarize(&self, request: &AssistantRequest) -> AssistantAnswer;

    /// Tags for the document, preferring `existing_tags`; the answer is [`AssistantAnswer::Tags`].
    fn suggest_tags(&self, request: &AssistantRequest, existing_tags: &IndexSet<TagName>) -> AssistantAnswer;

    /// Answers the last question of `conversation`, which alternates between the user and the AI.
    fn answer(&self, request: &AssistantRequest, conversation: &[ChatTurn]) -> AssistantAnswer;
}
