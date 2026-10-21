package dms.ui.service

import dms.application.assistant.AskAboutDocument
import dms.application.assistant.SuggestTags
import dms.application.assistant.SummarizeDocument
import dms.application.assistant.TagAllUntagged
import dms.application.classification.ApproveSuggestion
import dms.application.classification.ClassifyAllNew
import dms.application.classification.ListAiModels
import dms.application.classification.RejectSuggestion
import dms.application.classification.SuggestClassification

/** The AI's suggestions and their review, summaries and questions, and the connection test of the AI settings. */
class AiServices(
    val suggestClassification: SuggestClassification,
    val classifyAllNew: ClassifyAllNew,
    val approveSuggestion: ApproveSuggestion,
    val rejectSuggestion: RejectSuggestion,
    val listAiModels: ListAiModels,
    val summarizeDocument: SummarizeDocument,
    val suggestTags: SuggestTags,
    val tagAllUntagged: TagAllUntagged,
    val askAboutDocument: AskAboutDocument,
)
