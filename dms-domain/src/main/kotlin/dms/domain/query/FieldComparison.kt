package dms.domain.query

/**
 * How a [FieldConditionModel] compares a document's field value. [WITHIN_DAYS] takes a number of days and
 * matches dates from today up to that many days ahead, so a saved search stays current.
 */
enum class FieldComparison {
    PRESENT,
    ABSENT,
    EQUALS,
    CONTAINS,
    LESS_THAN,
    GREATER_THAN,
    WITHIN_DAYS,
}
