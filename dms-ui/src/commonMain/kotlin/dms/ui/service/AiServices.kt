package dms.ui.service

import dms.application.classification.ApproveSuggestion
import dms.application.classification.ClassifyAllNew
import dms.application.classification.ListAiModels
import dms.application.classification.RejectSuggestion
import dms.application.classification.SuggestClassification

/** The AI's suggestions and their review, and the connection test of the AI settings. */
class AiServices(
    val suggestClassification: SuggestClassification,
    val classifyAllNew: ClassifyAllNew,
    val approveSuggestion: ApproveSuggestion,
    val rejectSuggestion: RejectSuggestion,
    val listAiModels: ListAiModels,
)
