use std::sync::Arc;

use crate::assistant::{AskAboutDocument, SuggestTags, SummarizeDocument, TagAllUntagged};
use crate::classification::{ApproveSuggestion, ClassifyAllNew, ListAiModels, RejectSuggestion, SuggestClassification};
use crate::search::TranslateQuery;

/// The AI's suggestions and their review, summaries and questions, sentences turned into filters, and the connection test of the AI settings.
pub struct AiServices {
    pub suggest_classification: Arc<SuggestClassification>,
    pub classify_all_new: Arc<ClassifyAllNew>,
    pub approve_suggestion: Arc<ApproveSuggestion>,
    pub reject_suggestion: Arc<RejectSuggestion>,
    pub list_ai_models: Arc<ListAiModels>,
    pub summarize_document: Arc<SummarizeDocument>,
    pub suggest_tags: Arc<SuggestTags>,
    pub tag_all_untagged: Arc<TagAllUntagged>,
    pub ask_about_document: Arc<AskAboutDocument>,
    pub translate_query: Arc<TranslateQuery>,
}
