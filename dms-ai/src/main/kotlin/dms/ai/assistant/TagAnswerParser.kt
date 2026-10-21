package dms.ai.assistant

import dms.domain.taxonomy.TagName

/**
 * Understands the model's answer to "return only the tags, comma-separated". Models still add a
 * lead-in ("Tags:", "Here are the tags:"), code fences, bullets, numbering, quotes or hashes, or put
 * one tag per line; all of that is removed, and fragments longer than [MAX_WORDS] words are no tags.
 * Existing tags keep their spelling, and at most [MAX_TAGS] tags are kept.
 */
object TagAnswerParser {

    private const val MAX_TAGS = 8
    private const val MAX_WORDS = 3
    private val SEPARATORS = Regex(pattern = "[,;\\n]")
    private val LEADING = Regex(pattern = "^(?:[\\s\\-*•#>\"'`]+|\\d+[.)]\\s+)+")
    private val TRAILING = Regex(pattern = "[\\s\"'`.*]+$")

    fun parse(answer: String, existingTags: Set<TagName>): List<TagName> {
        val known = existingTags.associateBy { it }
        return answer.lines()
            .filterNot { it.trim().startsWith(prefix = "```") }
            .joinToString(separator = "\n") { it.substringAfter(delimiter = ':') }
            .split(SEPARATORS)
            .map { part -> TRAILING.replace(input = LEADING.replace(input = part, replacement = ""), replacement = "") }
            .filter { it.isNotEmpty() && it.split(' ').size <= MAX_WORDS }
            .mapNotNull { raw -> runCatching { TagName.of(raw = raw) }.getOrNull() }
            .map { known[it] ?: it }
            .distinct()
            .take(n = MAX_TAGS)
    }
}
