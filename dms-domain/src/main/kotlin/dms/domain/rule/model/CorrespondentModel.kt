package dms.domain.rule.model

import java.util.Locale

/**
 * Someone the library receives documents from. It is recognized in a document's text by its [name]
 * or one of its [aliases], compared as whole words and ignoring case.
 */
data class CorrespondentModel(
    val name: String,
    val aliases: List<String> = emptyList(),
) {
    init {
        require(value = name.isNotBlank()) { "Correspondent name must not be blank" }
        require(value = aliases.none { it.isBlank() }) { "Aliases must not be blank" }
    }

    /** The name ignoring case, which identifies the correspondent. */
    val key: String
        get() {
            return name.trim().lowercase(Locale.ROOT)
        }

    /** `true` if the name or an alias appears in [text] as a whole word or phrase. */
    fun appearsIn(text: String): Boolean {
        val haystack = text.lowercase(Locale.ROOT)
        return (listOf(name) + aliases).any { containsPhrase(haystack = haystack, phrase = it) }
    }

    private fun containsPhrase(haystack: String, phrase: String): Boolean {
        val needle = phrase.trim().lowercase(Locale.ROOT)
        var index = haystack.indexOf(string = needle)
        while (index >= 0) {
            val before = haystack.getOrNull(index = index - 1)
            val after = haystack.getOrNull(index = index + needle.length)
            if (!isWordCharacter(character = before) && !isWordCharacter(character = after)) {
                return true
            }
            index = haystack.indexOf(string = needle, startIndex = index + 1)
        }
        return false
    }

    private fun isWordCharacter(character: Char?): Boolean {
        return character != null && character.isLetterOrDigit()
    }
}
