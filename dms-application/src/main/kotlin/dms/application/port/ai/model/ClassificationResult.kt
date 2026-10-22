package dms.application.port.ai.model

import dms.domain.classification.model.ClassificationSuggestionModel

sealed interface ClassificationResult {

    data class Suggested(val suggestion: ClassificationSuggestionModel) : ClassificationResult

    data class Failed(val reason: ClassificationFailure, val detail: String? = null) : ClassificationResult
}
