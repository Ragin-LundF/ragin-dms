package dms.domain.field

import dms.domain.field.model.CurrencyCodeModel
import java.math.BigDecimal
import java.time.LocalDate

/** The value of a custom field on one document. Its [type] must match the field's definition. */
sealed interface FieldValue {

    val type: FieldType

    data class Text(val text: String) : FieldValue {
        init {
            require(value = text.isNotBlank()) { "A text value must not be blank" }
        }

        override val type: FieldType
            get() {
                return FieldType.TEXT
            }
    }

    data class Number(val number: BigDecimal) : FieldValue {
        override val type: FieldType
            get() {
                return FieldType.NUMBER
            }
    }

    data class Money(val amount: BigDecimal, val currency: CurrencyCodeModel) : FieldValue {
        override val type: FieldType
            get() {
                return FieldType.MONEY
            }
    }

    data class Date(val date: LocalDate) : FieldValue {
        override val type: FieldType
            get() {
                return FieldType.DATE
            }
    }

    data class Flag(val flag: Boolean) : FieldValue {
        override val type: FieldType
            get() {
                return FieldType.BOOLEAN
            }
    }
}
