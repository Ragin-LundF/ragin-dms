package dms.domain.field.model

import java.util.Locale

/**
 * Identifies a custom field across the library, for example `amount` or `due`. Lowercase letters,
 * digits, `-` and `_`, starting with a letter, so it reads well in the catalog file and in path
 * templates (`{field:due}`).
 */
@JvmInline
value class FieldKeyModel(val value: String) {
    init {
        require(value = PATTERN.matches(input = value)) { "Not a valid field key: $value" }
    }

    override fun toString(): String {
        return value
    }

    companion object {
        const val MAX_LENGTH: Int = 40

        private val PATTERN = Regex(pattern = "^[a-z][a-z0-9_-]{0,${MAX_LENGTH - 1}}$")
        private val SEPARATORS = Regex(pattern = "[\\s_]+")
        private val NOT_ALLOWED = Regex(pattern = "[^a-z0-9_-]")

        /** A key derived from a label such as `Contract end`, or `null` if nothing usable is left. */
        fun fromLabel(label: String): FieldKeyModel? {
            val key = label.trim()
                .lowercase(Locale.ROOT)
                .replace(regex = SEPARATORS, replacement = "-")
                .replace(regex = NOT_ALLOWED, replacement = "")
                .trimStart('-', '_')
                .dropLastWhile { it == '-' }
                .take(n = MAX_LENGTH)
            return key.takeIf { PATTERN.matches(input = it) }?.let { FieldKeyModel(value = it) }
        }
    }
}
