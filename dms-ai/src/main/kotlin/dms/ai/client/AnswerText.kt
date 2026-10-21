package dms.ai.client

/**
 * Cleans what a model answered: reasoning models put their thoughts in `<think>` blocks before the
 * answer, which the user should not see, and some models mark words **bold** despite being asked not to.
 */
object AnswerText {

    private val THINKING = Regex(pattern = "<think>.*?</think>", option = RegexOption.DOT_MATCHES_ALL)

    private val EMPHASIS = Regex(pattern = "\\*\\*")

    /** The answer without Markdown emphasis, which the app shows as plain text. */
    fun plain(content: String): String {
        return EMPHASIS.replace(input = clean(content = content), replacement = "")
    }

    fun clean(content: String): String {
        val withoutThoughts = THINKING.replace(input = content, replacement = "")
        return withoutThoughts.substringAfter(delimiter = "</think>").trim()
    }
}
