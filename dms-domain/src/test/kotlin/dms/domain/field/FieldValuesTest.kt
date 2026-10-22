package dms.domain.field

import dms.domain.field.model.CurrencyCodeModel
import dms.domain.field.model.FieldDefinitionModel
import dms.domain.field.model.FieldKeyModel
import java.math.BigDecimal
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class FieldValuesTest {

    private val euro = CurrencyCodeModel(value = "EUR")

    @Test
    fun `money takes the currency it names, before or after the amount`() {
        assertEquals(
            expected = FieldValue.Money(amount = BigDecimal("1234.50"), currency = CurrencyCodeModel(value = "CHF")),
            actual = FieldValues.parse(type = FieldType.MONEY, raw = "CHF 1'234.50", defaultCurrency = euro),
        )
        assertEquals(
            expected = FieldValue.Money(amount = BigDecimal("1234.5"), currency = CurrencyCodeModel(value = "USD")),
            actual = FieldValues.parse(type = FieldType.MONEY, raw = "1,234.5 usd", defaultCurrency = euro),
        )
    }

    @Test
    fun `money without a currency takes the library's default, and without one it is refused`() {
        assertEquals(
            expected = FieldValue.Money(amount = BigDecimal("99.90"), currency = euro),
            actual = FieldValues.parse(type = FieldType.MONEY, raw = "99,90", defaultCurrency = euro),
        )
        assertNull(actual = FieldValues.parse(type = FieldType.MONEY, raw = "99,90", defaultCurrency = null))
    }

    @Test
    fun `values that do not fit the type are refused`() {
        assertNull(actual = FieldValues.parse(type = FieldType.NUMBER, raw = "twelve", defaultCurrency = null))
        assertNull(actual = FieldValues.parse(type = FieldType.DATE, raw = "31.02.2026", defaultCurrency = null))
        assertNull(actual = FieldValues.parse(type = FieldType.BOOLEAN, raw = "maybe", defaultCurrency = null))
        assertNull(actual = FieldValues.parse(type = FieldType.TEXT, raw = "   ", defaultCurrency = null))
    }

    @Test
    fun `every type reads back what format writes`() {
        val values = listOf(
            FieldValue.Text(text = "Contract 7"),
            FieldValue.Number(number = BigDecimal("-3.25")),
            FieldValue.Money(amount = BigDecimal("1000.00"), currency = euro),
            FieldValue.Date(date = LocalDate.of(2026, 12, 1)),
            FieldValue.Flag(flag = true),
        )

        values.forEach { value ->
            val text = FieldValues.format(value = value)
            assertEquals(
                expected = value,
                actual = FieldValues.parse(type = value.type, raw = text, defaultCurrency = null),
            )
        }
    }

    @Test
    fun `keys are derived from labels`() {
        assertEquals(
            expected = FieldKeyModel(value = "contract-end"),
            actual = FieldKeyModel.fromLabel(label = "Contract end"),
        )
        assertEquals(expected = FieldKeyModel(value = "betrag"), actual = FieldKeyModel.fromLabel(label = " Betrag! "))
        assertNull(actual = FieldKeyModel.fromLabel(label = "123"))
    }

    @Test
    fun `only date fields can be deadlines`() {
        kotlin.test.assertFailsWith<IllegalArgumentException> {
            FieldDefinitionModel(
                key = FieldKeyModel(value = "x"),
                label = "X",
                type = FieldType.TEXT,
                role = FieldRole.DUE_DATE,
            )
        }
    }
}
