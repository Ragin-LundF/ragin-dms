package dms.domain.query.model

import dms.domain.field.FieldValue
import dms.domain.field.FieldValues
import dms.domain.field.model.FieldKeyModel
import dms.domain.query.FieldComparison
import java.math.BigDecimal
import java.time.LocalDate
import java.util.Locale

/**
 * One condition on a custom field, such as `amount > 100` or `due before 2026-12-01`. [value] is
 * needed by every comparison except [FieldComparison.PRESENT] and [FieldComparison.ABSENT]. Values of
 * different types never match, except that a number compares with a money amount.
 */
data class FieldConditionModel(
    val key: FieldKeyModel,
    val comparison: FieldComparison,
    val value: FieldValue? = null,
) {
    init {
        val needsValue = comparison != FieldComparison.PRESENT && comparison != FieldComparison.ABSENT
        require(value = !needsValue || value != null) { "$comparison needs a value" }
        require(value = comparison != FieldComparison.WITHIN_DAYS || value is FieldValue.Number) {
            "WITHIN_DAYS needs a number of days"
        }
    }

    fun matches(fields: Map<FieldKeyModel, FieldValue>, today: LocalDate): Boolean {
        val actual = fields[key]
        return when (comparison) {
            FieldComparison.PRESENT -> actual != null
            FieldComparison.ABSENT -> actual == null
            else -> actual != null && compare(actual = actual, today = today)
        }
    }

    private fun compare(actual: FieldValue, today: LocalDate): Boolean {
        val expected = checkNotNull(value = value)
        return when (comparison) {
            FieldComparison.EQUALS -> order(actual = actual, expected = expected) == 0
            FieldComparison.LESS_THAN -> (order(actual = actual, expected = expected) ?: 0) < 0
            FieldComparison.GREATER_THAN -> (order(actual = actual, expected = expected) ?: 0) > 0
            FieldComparison.CONTAINS -> textOf(value = actual).contains(other = textOf(value = expected))
            FieldComparison.WITHIN_DAYS -> withinDays(actual = actual, days = expected, today = today)
            FieldComparison.PRESENT, FieldComparison.ABSENT -> true
        }
    }

    private fun withinDays(actual: FieldValue, days: FieldValue, today: LocalDate): Boolean {
        val date = (actual as? FieldValue.Date)?.date ?: return false
        val count = (days as FieldValue.Number).number.toLong()
        return !date.isBefore(today) && !date.isAfter(today.plusDays(count))
    }

    /** Negative, zero or positive like `compareTo`, or `null` if the two values cannot be compared. */
    private fun order(actual: FieldValue, expected: FieldValue): Int? {
        val actualNumber = numberOf(value = actual)
        val expectedNumber = numberOf(value = expected)
        return when {
            actualNumber != null && expectedNumber != null -> actualNumber.compareTo(other = expectedNumber)
            actual is FieldValue.Date && expected is FieldValue.Date -> actual.date.compareTo(other = expected.date)
            actual is FieldValue.Flag && expected is FieldValue.Flag -> actual.flag.compareTo(other = expected.flag)
            actual is FieldValue.Text && expected is FieldValue.Text ->
                textOf(value = actual).compareTo(other = textOf(value = expected))

            else -> null
        }
    }

    private fun numberOf(value: FieldValue): BigDecimal? {
        return when (value) {
            is FieldValue.Number -> value.number
            is FieldValue.Money -> value.amount
            else -> null
        }
    }

    private fun textOf(value: FieldValue): String {
        return FieldValues.format(value = value).lowercase(Locale.ROOT)
    }
}
