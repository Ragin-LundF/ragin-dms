package dms.domain.rule.match

/** Which part of a suggestion a rule decided; see [RuleResultModel.sources]. */
enum class RuleTarget {
    CATEGORY,
    TAGS,
    FIELDS,
    TARGET_FOLDER,
}
