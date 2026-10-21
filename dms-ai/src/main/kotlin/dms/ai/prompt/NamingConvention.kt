package dms.ai.prompt

/**
 * Describes how the library names its files, read from existing file names, so the model's file
 * name fits in. Nothing about the convention is hard-coded; with too few examples it says so.
 */
object NamingConvention {

    private const val MIN_EXAMPLES = 3
    private const val MAJORITY = 0.5
    private val DATE_PREFIXES = listOf(
        Regex(pattern = "^\\d{4}-\\d{2}-\\d{2}") to "start with the document date as YYYY-MM-DD",
        Regex(pattern = "^\\d{4}-\\d{2}(?!-\\d)") to "start with year and month as YYYY-MM",
        Regex(pattern = "^\\d{8}") to "start with the document date as YYYYMMDD",
        Regex(pattern = "^\\d{4}[ _-]") to "start with the year",
    )
    private val SEPARATORS = listOf(" - " to "\" - \"", "_" to "underscores", " " to "spaces")

    fun describe(examples: List<String>): String {
        val names = examples.map { it.substringBeforeLast(delimiter = '.') }.filter { it.isNotBlank() }
        if (names.size < MIN_EXAMPLES) {
            return "The library has too few files to show a convention. Use a short, descriptive name."
        }
        val parts = mutableListOf<String>()
        DATE_PREFIXES.firstOrNull { (pattern, _) -> share(names = names) { pattern.containsMatchIn(input = it) } }
            ?.let { (_, description) -> parts += description }
        SEPARATORS.firstOrNull { (separator, _) -> share(names = names) { separator in it } }
            ?.let { (_, description) -> parts += "separate their parts with $description" }
        if (parts.isEmpty()) {
            return "No common pattern; follow the examples."
        }
        return "Most file names ${parts.joinToString(separator = " and ")}."
    }

    private fun share(names: List<String>, predicate: (String) -> Boolean): Boolean {
        return names.count(predicate = predicate) > names.size * MAJORITY
    }
}
