package dms.ai.assistant

import dms.ai.prompt.PromptTemplates
import dms.ai.prompt.PromptTexts
import dms.ai.prompt.TaxTag
import dms.application.port.ai.model.DocumentContent
import dms.application.port.assistant.model.AssistantRequest
import dms.domain.taxonomy.TagName

/** The prompts of the document assistant, from the templates in `resources/prompts`. */
object AssistantPrompts {

    private const val MAX_LISTED = 300

    fun summary(): String {
        return PromptTemplates.fill(name = "summary.md", values = emptyMap())
    }

    fun summaryRequest(request: AssistantRequest): String {
        return PromptTemplates.fill(name = "summary-request.md", values = documentValues(request = request))
    }

    fun tags(existingTags: Set<TagName>): String {
        val names = existingTags.map { it.value }.sorted()
        val listed = if (names.isEmpty()) "none yet" else names.take(n = MAX_LISTED).joinToString(separator = ", ")
        return PromptTemplates.fill(
            name = "tags.md",
            values = mapOf("taxTag" to TaxTag.of(existingTags = names), "tags" to listed),
        )
    }

    fun tagsRequest(request: AssistantRequest): String {
        return PromptTemplates.fill(name = "tags-request.md", values = documentValues(request = request))
    }

    /** The system message of a conversation. It holds the document's text; page images go with the first question. */
    fun chat(request: AssistantRequest): String {
        val note = if (request.content is DocumentContent.PageImages) {
            "The document is attached as page images to the first question."
        } else {
            "The text of the document follows."
        }
        return PromptTemplates.fill(
            name = "chat.md",
            values = mapOf("documentNote" to note) + documentValues(request = request),
        )
    }

    private fun documentValues(request: AssistantRequest): Map<String, String> {
        val text = (request.content as? DocumentContent.Text)?.text
        return mapOf("fileName" to request.fileName, "document" to PromptTexts.documentSection(text = text))
    }
}
