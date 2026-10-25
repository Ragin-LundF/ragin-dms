package dms.ai.prompt

import dms.application.port.ai.model.ClassificationRequestModel
import dms.domain.field.FieldType
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel

/**
 * The words sent to the model, from the templates in `resources/prompts`. The system message holds
 * the rules and the library's taxonomy, the user message the document. The model gets the existing
 * names, so it prefers what the library uses.
 */
object PromptTexts {

    private const val MAX_LISTED = 200

    fun system(request: ClassificationRequestModel): String {
        val extension = request.currentFileName.substringAfterLast(delimiter = '.', missingDelimiterValue = "")
        val categories = request.categories.map { pathOf(category = it, categories = request.categories) }.sorted()
        return PromptTemplates.fill(
            name = "classification.md",
            values = mapOf(
                "extension" to extension,
                "categories" to listed(values = categories),
                "tags" to listed(values = request.tags.map { it.value }.sorted()),
                "taxTag" to TaxTag.of(existingTags = request.tags.map { it.value }),
                "fields" to fieldsOf(request = request),
                "convention" to NamingConvention.describe(examples = request.fileNameExamples),
                "examples" to listed(values = request.fileNameExamples),
                "today" to request.today.toString(),
            ),
        )
    }

    fun user(request: ClassificationRequestModel, text: String?): String {
        val folders = listed(values = request.folders.map { it.value })
        return PromptTemplates.fill(
            name = "classification-request.md",
            values = mapOf(
                "fileName" to request.currentFileName,
                "folders" to folders,
                "outline" to outlineSection(request = request),
                "rules" to rulesSection(request = request),
                "document" to documentSection(text = text),
            ),
        )
    }

    /**
     * The document's text, already cut to the configured input budget when it was read, or a note that
     * `null` text means page images.
     */
    fun documentSection(text: String?): String {
        if (text == null) {
            return PromptTemplates.fill(name = "document-images.md", values = emptyMap())
        }
        return PromptTemplates.fill(
            name = "document-text.md",
            values = mapOf("text" to text),
        )
    }

    /**
     * The folder outline for a target-folder suggestion, one folder per line with its document count,
     * main category and sample titles; or a note that no target folder is wanted.
     */
    fun outlineSection(request: ClassificationRequestModel): String {
        val outline = request.outline
            ?: return PromptTemplates.fill(name = "classification-outline-none.md", values = emptyMap())
        if (outline.folders.isEmpty()) {
            return PromptTemplates.fill(name = "classification-outline-empty.md", values = emptyMap())
        }
        val names = request.categories.associateBy { it.id }
        val lines = outline.folders.map { folder ->
            val category = folder.topCategory?.let { names[it] }?.let {
                pathOf(category = it, categories = request.categories)
            }
            listOfNotNull(
                folder.path.value,
                "${folder.documentCount} documents",
                category?.let { "mostly $it" },
                folder.sampleTitles.takeIf { it.isNotEmpty() }?.let { "e.g. ${it.joinToString(separator = ", ")}" },
                folder.hiddenSubfolders.takeIf { it > 0 }?.let { "$it more subfolders" },
            ).joinToString(separator = " | ")
        }
        return PromptTemplates.fill(
            name = "classification-outline.md",
            values = mapOf("folders" to lines.joinToString(separator = "\n")),
        )
    }

    /** What the user's matching rules decided; the model is asked to keep it, and it overrides the answer. */
    fun rulesSection(request: ClassificationRequestModel): String {
        val rules = request.fixedValues ?: return ""
        val category = rules.categoryId?.let { id -> request.categories.firstOrNull { it.id == id } }
            ?.let { pathOf(category = it, categories = request.categories) }
        val parts = listOfNotNull(
            category?.let { "category: $it" },
            rules.correspondent?.let { "correspondent: $it" },
            rules.tags.takeIf { it.isNotEmpty() }?.let { tags -> "tags: ${tags.joinToString { it.value }}" },
        )
        if (parts.isEmpty()) {
            return ""
        }
        return PromptTemplates.fill(
            name = "classification-rules.md",
            values = mapOf("values" to parts.joinToString(separator = "; ")),
        )
    }

    /** The custom fields as `key (Label, TYPE)`, with the default currency for money fields. */
    fun fieldsOf(request: ClassificationRequestModel): String {
        val currency = request.defaultCurrency?.let { " in $it unless stated otherwise" }.orEmpty()
        return listed(
            values = request.fieldDefinitions.map { definition ->
                val type = if (definition.type == FieldType.MONEY) "MONEY$currency" else definition.type.name
                "${definition.key} (${definition.label}, $type)"
            },
        )
    }

    /** "Parent / Child" for nested categories, the way the user sees them. */
    fun pathOf(category: CategoryModel, categories: List<CategoryModel>): String {
        val byId: Map<CategoryIdModel, CategoryModel> = categories.associateBy { it.id }
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
