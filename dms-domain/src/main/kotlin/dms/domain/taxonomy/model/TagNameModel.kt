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

        /** The tag for [raw]; throws [IllegalArgumentException] for a blank or too long one. */
        fun of(raw: String): TagNameModel {
            val cleaned = clean(raw = raw)
            require(value = cleaned.isNotEmpty()) { "Tag must not be blank" }
            require(value = cleaned.length <= MAX_LENGTH) { "Tag is longer than $MAX_LENGTH characters" }
            return TagNameModel(value = cleaned)
        }

        /** The tag for text typed by a user or written by the AI, or `null` if it is blank or too long. */
        fun parse(raw: String): TagNameModel? {
            val cleaned = clean(raw = raw)
            if (cleaned.isEmpty() || cleaned.length > MAX_LENGTH) {
                return null
            }
            return TagNameModel(value = cleaned)
        }

        private fun clean(raw: String): String {
            return Normalizer.normalize(raw, Normalizer.Form.NFC)
                .trim()
                .replace(regex = WHITESPACE, replacement = " ")
        }
    }
}
