package dms.ai.prompt

import java.util.Locale

/** How the prompts name the tag for documents that matter for taxes: the library's own, if it has one. */
object TaxTag {

    private val NAMES = listOf("steuer", "tax")

    fun of(existingTags: Collection<String>): String {
        val existing = existingTags.firstOrNull { tag ->
            val name = tag.lowercase(Locale.ROOT)
            NAMES.any { it in name }
        }
        return if (existing != null) "\"$existing\"" else "\"Tax\" (\"Steuer\" if you write German tags)"
    }
}
