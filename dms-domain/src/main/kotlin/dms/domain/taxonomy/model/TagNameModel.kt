package dms.domain.taxonomy.model

import java.text.Normalizer
import java.util.Locale

/**
 * A tag, normalized once on creation: NFC, trimmed, inner whitespace collapsed.
 *
 * Tags are compared case-insensitively but keep the casing they were created with, so `Tax` and
 * `tax` are the same tag and only one of them is ever stored.
 */
class TagNameModel private constructor(val value: String) : Comparable<TagNameModel> {

    private val key: String = value.lowercase(Locale.ROOT)

    override fun equals(other: Any?): Boolean {
        return other is TagNameModel && other.key == key
    }

    override fun hashCode(): Int {
        return key.hashCode()
    }

    override fun compareTo(other: TagNameModel): Int {
        return key.compareTo(other = other.key)
    }

    override fun toString(): String {
        return value
    }

    companion object {
        const val MAX_LENGTH: Int = 60

        private val WHITESPACE = Regex(pattern = "\\s+")

        fun of(raw: String): TagNameModel {
            val cleaned = Normalizer.normalize(raw, Normalizer.Form.NFC)
                .trim()
                .replace(regex = WHITESPACE, replacement = " ")
            require(value = cleaned.isNotEmpty()) { "Tag must not be blank" }
            require(value = cleaned.length <= MAX_LENGTH) { "Tag is longer than $MAX_LENGTH characters" }
            return TagNameModel(value = cleaned)
        }
    }
}
