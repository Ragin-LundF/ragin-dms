package dms.domain.field

/**
 * Marks a date field as a deadline, which the dashboard announces [defaultLeadDays] days ahead.
 * Retention is no role: it is strictly opt-in and kept apart from the fields.
 */
enum class FieldRole(val defaultLeadDays: Int) {
    DUE_DATE(defaultLeadDays = 7),
    CANCEL_BY(defaultLeadDays = 30),
    CONTRACT_END(defaultLeadDays = 30),
    WARRANTY_END(defaultLeadDays = 30),
}
