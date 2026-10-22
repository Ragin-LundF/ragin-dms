package dms.domain.field.model

import java.util.Locale

/**
 * An ISO 4217 currency code such as `EUR`. The library's default currency is a setting the user
 * chooses; no currency is assumed anywhere in the logic.
 */
@JvmInline
value class CurrencyCodeModel(val value: String) {
    init {
        require(value = PATTERN.matches(input = value)) { "Not an ISO 4217 currency code: $value" }
    }

    override fun toString(): String {
        return value
    }

    companion object {
        private val PATTERN = Regex(pattern = "^[A-Z]{3}$")

        /** The code in [raw], ignoring case and surrounding blanks, or `null` if it is none. */
        fun parse(raw: String): CurrencyCodeModel? {
            val code = raw.trim().uppercase(Locale.ROOT)
            return code.takeIf { PATTERN.matches(input = it) }?.let { CurrencyCodeModel(value = it) }
        }
    }
}
