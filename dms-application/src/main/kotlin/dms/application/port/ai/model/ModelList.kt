package dms.application.port.ai.model

/** The answer of an AI server to "which models do you have?". */
sealed interface ModelList {

    data class Available(val models: List<String>) : ModelList

    data class Unavailable(val reason: ClassificationFailure, val detail: String? = null) : ModelList
}
