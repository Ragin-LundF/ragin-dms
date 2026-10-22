package dms.domain.document.model

import java.util.Locale

/**
 * SHA-256 of a file's content as 64 lowercase hex characters. It is how a moved or renamed file is
 * recognized again, and how two library folders are compared without trusting modification times.
 */
@JvmInline
value class ContentHashModel(val value: String) {
    init {
        require(value = SHA256_HEX.matches(input = value)) { "Not a lowercase SHA-256 hex digest: $value" }
    }

    override fun toString(): String {
        return value
    }

    companion object {
        private val SHA256_HEX = Regex(pattern = "^[0-9a-f]{64}$")

        fun fromHex(hex: String): ContentHashModel {
            return ContentHashModel(value = hex.lowercase(Locale.ROOT))
        }
    }
}
