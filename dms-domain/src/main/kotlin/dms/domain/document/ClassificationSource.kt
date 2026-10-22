package dms.domain.document

/** Who produced the categorization the user confirmed last: the AI, the user, or the matching rules. */
enum class ClassificationSource {
    AI,
    MANUAL,
    RULE,
}
