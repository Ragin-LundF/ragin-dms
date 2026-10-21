package dms.ai.prompt

import dms.application.port.ai.model.ClassificationRequest
import dms.domain.taxonomy.Category
import dms.domain.taxonomy.CategoryId

/**
 * The words sent to the model, from the templates in `resources/prompts`. The system message holds
 * the rules and the library's taxonomy, the user message the document. The model gets the existing
 * names, so it prefers what the library uses.
 */
object PromptTexts {

    /** Enough text for a classification; longer documents are cut. */
    const val MAX_TEXT_CHARACTERS: Int = 12_000
    private const val MAX_LISTED = 200
    private val SYSTEM: String by lazy { resource(name = "system.md") }
    private val USER: String by lazy { resource(name = "user.md") }

    fun system(request: ClassificationRequest): String {
        val extension = request.currentFileName.substringAfterLast(delimiter = '.', missingDelimiterValue = "")
        val categories = request.categories.map { pathOf(category = it, categories = request.categories) }.sorted()
        return fill(
            template = SYSTEM,
            values = mapOf(
                "extension" to extension,
                "categories" to listed(values = categories),
                "tags" to listed(values = request.tags.map { it.value }.sorted()),
                "convention" to NamingConvention.describe(examples = request.fileNameExamples),
                "examples" to listed(values = request.fileNameExamples),
                "today" to request.today.toString(),
            ),
        )
    }

    fun user(request: ClassificationRequest, text: String?): String {
        val folders = listed(values = request.folders.map { it.value })
        val document = if (text == null) {
            "The document is attached as page images."
        } else {
            "Text of the document (long documents are cut after $MAX_TEXT_CHARACTERS characters):\n" +
                text.take(n = MAX_TEXT_CHARACTERS)
        }
        return fill(
            template = USER,
            values = mapOf("fileName" to request.currentFileName, "folders" to folders, "document" to document),
        )
    }

    /** "Parent / Child" for nested categories, the way the user sees them. */
    fun pathOf(category: Category, categories: List<Category>): String {
        val byId: Map<CategoryId, Category> = categories.associateBy { it.id }
        val names = generateSequence(seed = category) { current -> current.parentId?.let { byId[it] } }
            .take(n = categories.size)
            .map { it.name }
            .toList()
        return names.reversed().joinToString(separator = " / ")
    }

    private fun fill(template: String, values: Map<String, String>): String {
        return values.entries.fold(initial = template) { text, (key, value) -> text.replace("{{$key}}", value) }
    }

    private fun resource(name: String): String {
        val stream = checkNotNull(value = PromptTexts::class.java.getResourceAsStream("/prompts/$name")) {
            "Missing prompt resource $name"
        }
        return stream.use { it.readBytes().decodeToString() }.trim()
    }

    private fun listed(values: List<String>): String {
        return if (values.isEmpty()) "none yet" else values.take(n = MAX_LISTED).joinToString(separator = "; ")
    }
}
