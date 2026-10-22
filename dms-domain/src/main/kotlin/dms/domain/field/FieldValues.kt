package dms.domain.field

import dms.domain.field.model.CurrencyCodeModel
import java.math.BigDecimal
import java.time.LocalDate
import java.time.format.DateTimeParseException
import java.util.Locale

/**
 * Reads and writes field values as plain text, the way the user types them and the AI answers.
 * A value that does not fit the field's type is refused with `null`, never guessed.
 */
object FieldValues {

    private val TRUE_WORDS = setOf("true", "yes", "1")
    private val FALSE_WORDS = setOf("false", "no", "0")
    private val AMOUNT_WITH_CURRENCY = Regex(pattern = "^([A-Za-z]{3})?\\s*([-+]?[0-9][0-9.,']*)\\s*([A-Za-z]{3})?$")

    /** [raw] as a value of [type]; a money value without a currency takes [defaultCurrency]. */
    fun parse(type: FieldType, raw: String, defaultCurrency: CurrencyCodeModel?): FieldValue? {
        val text = raw.trim()
        if (text.isEmpty()) {
            return null
        }
        return when (type) {
            FieldType.TEXT -> FieldValue.Text(text = text)
            FieldType.NUMBER -> number(raw = text)?.let { FieldValue.Number(number = it) }
            FieldType.MONEY -> money(raw = text, defaultCurrency = defaultCurrency)
            FieldType.DATE -> date(raw = text)?.let { FieldValue.Date(date = it) }
            FieldType.BOOLEAN -> flag(raw = text)?.let { FieldValue.Flag(flag = it) }
        }
    }

    /** The value as plain text that [parse] reads back: `1234.50 EUR`, `2026-12-01`, `true`. */
    fun format(value: FieldValue): String {
        return when (value) {
            is FieldValue.Text -> value.text
            is FieldValue.Number -> value.number.toPlainString()
            is FieldValue.Money -> "${value.amount.toPlainString()} ${value.currency}"
            is FieldValue.Date -> value.date.toString()
            is FieldValue.Flag -> value.flag.toString()
        }
    }

    /** A plain number; both `1234.5` and `1.234,5` are understood, with the last separator as decimal point. */
    fun number(raw: String): BigDecimal? {
        val cleaned = raw.trim().replace(oldValue = "'", newValue = "").replace(oldValue = " ", newValue = "")
        val lastDot = cleaned.lastIndexOf(char = '.')
        val lastComma = cleaned.lastIndexOf(char = ',')
        val decimal = when {
            lastDot >= 0 && lastComma >= 0 -> if (lastDot > lastComma) '.' else ','
            lastComma >= 0 && cleaned.count { it == ',' } == 1 -> ','
            else -> '.'
        }
        val grouping = if (decimal == '.') ',' else '.'
        val normalized = cleaned.replace(oldValue = grouping.toString(), newValue = "")
            .replace(oldChar = decimal, newChar = '.')
        return normalized.toBigDecimalOrNull()
    }

    private fun money(raw: String, defaultCurrency: CurrencyCodeModel?): FieldValue.Money? {
        val match = AMOUNT_WITH_CURRENCY.matchEntire(input = raw) ?: return null
        val (before, amountText, after) = match.destructured
        if (before.isNotEmpty() && after.isNotEmpty()) {
            return null
        }
        val currency = CurrencyCodeModel.parse(raw = before.ifEmpty { after }) ?: defaultCurrency ?: return null
        val amount = number(raw = amountText) ?: return null
        return FieldValue.Money(amount = amount, currency = currency)
    }

    private fun date(raw: String): LocalDate? {
        return try {
            LocalDate.parse(raw)
        } catch (ignored: DateTimeParseException) {
            null
        }
    }

    private fun flag(raw: String): Boolean? {
        val word = raw.lowercase(Locale.ROOT)
        return when (word) {
            in TRUE_WORDS -> true
            in FALSE_WORDS -> false
            else -> null
        }
    }
}
