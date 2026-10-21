package dms.application.port.ai.model

import dms.domain.classification.ClassificationSuggestion

sealed interface ClassificationResult {

    data class Suggested(val suggestion: ClassificationSuggestion) : ClassificationResult

    data class Failed(val reason: ClassificationFailure, val detail: String? = null) : ClassificationResult
}
