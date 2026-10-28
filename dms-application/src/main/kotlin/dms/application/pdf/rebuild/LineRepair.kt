package dms.application.pdf.rebuild

/**
 * Some models write NUL or U+FFFD instead of a character (seen with umlauts under a JSON schema). A line
 * read again is taken only if it agrees with every character the first reading did get, so the second
 * reading can fill the gaps but never change the line.
 */
object LineRepair {

    private val LOST = Regex(pattern = "[\\u0000\\uFFFD]")
    private val LOST_RUN = Regex(pattern = "[\\u0000\\uFFFD]+")
    private val WHITESPACE = Regex(pattern = "\\s+")

    fun hasLostCharacters(text: String): Boolean {
        return LOST.containsMatchIn(input = text)
    }

    /** The repaired line, or `null` if [reread] does not fit [text]. */
    fun repaired(text: String, reread: String): String? {
        val candidate = normalized(text = reread)
        return candidate.takeIf { pattern(text = text).matches(input = it) }
    }

    /** Known characters must match exactly; each run of n lost characters may be 1..n other characters. */
    private fun pattern(text: String): Regex {
        val normal = normalized(text = text)
        val parts = LOST_RUN.split(input = normal)
        val runs = LOST_RUN.findAll(input = normal).map { it.value.length }.toList()
        val source = buildString {
            parts.forEachIndexed { i, part ->
                append(Regex.escape(literal = part))
                runs.getOrNull(index = i)?.let { append("[^\\s\\u0000\\uFFFD]{1,$it}") }
            }
        }
        return Regex(pattern = source)
    }

    private fun normalized(text: String): String {
        return text.trim().replace(regex = WHITESPACE, replacement = " ")
    }
}
