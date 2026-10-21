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

    fun system(request: ClassificationRequest): String {
        val extension = request.currentFileName.substringAfterLast(delimiter = '.', missingDelimiterValue = "")
        val categories = request.categories.map { pathOf(category = it, categories = request.categories) }.sorted()
        return PromptTemplates.fill(
            name = "classification.md",
            values = mapOf(
                "extension" to extension,
                "categories" to listed(values = categories),
                "tags" to listed(values = request.tags.map { it.value }.sorted()),
                "taxTag" to TaxTag.of(existingTags = request.tags.map { it.value }),
                "convention" to NamingConvention.describe(examples = request.fileNameExamples),
                "examples" to listed(values = request.fileNameExamples),
                "today" to request.today.toString(),
            ),
        )
    }

    fun user(request: ClassificationRequest, text: String?): String {
        val folders = listed(values = request.folders.map { it.value })
        return PromptTemplates.fill(
            name = "classification-request.md",
            values = mapOf(
                "fileName" to request.currentFileName,
                "folders" to folders,
                "document" to documentSection(text = text),
            ),
        )
    }

    /** The document's text, cut to [MAX_TEXT_CHARACTERS], or a note that `null` text means page images. */
    fun documentSection(text: String?): String {
        return if (text == null) {
            "The document is attached as page images."
        } else {
            "Text of the document (long documents are cut after $MAX_TEXT_CHARACTERS characters):\n" +
                text.take(n = MAX_TEXT_CHARACTERS)
        }
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

    private fun listed(values: List<String>): String {
        return if (values.isEmpty()) "none yet" else values.take(n = MAX_LISTED).joinToString(separator = "; ")
    }
}
