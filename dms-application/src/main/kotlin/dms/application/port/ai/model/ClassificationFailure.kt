package dms.application.port.ai.model

/** Why the classifier could not produce a suggestion. */
enum class ClassificationFailure {
    PROVIDER_UNAVAILABLE,
    MODEL_CANNOT_READ_IMAGES,
    INVALID_RESPONSE,
    TIMEOUT,
}
