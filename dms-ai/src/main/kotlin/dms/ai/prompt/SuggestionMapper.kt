package dms.ai.prompt

import dms.ai.client.dto.SuggestionDto
import dms.application.port.ai.model.ClassificationRequest
import dms.domain.classification.ClassificationSuggestion
import dms.domain.classification.SuggestedCategory
import dms.domain.classification.SuggestedTag
import dms.domain.library.FileNamePolicy
import dms.domain.taxonomy.TagName
import java.time.LocalDate
import java.time.format.DateTimeParseException
import java.util.Locale

/**
 * Turns the model's answer into a suggestion the domain accepts. A value that does not fit is dropped
 * rather than failing the whole suggestion: an unknown date format, an invalid file name, a blank tag.
 */
object SuggestionMapper {

    private const val MAX_TAGS = 8
    private val EXTENSION = Regex(pattern = "[A-Za-z0-9]{1,5}")

    fun toSuggestion(answer: SuggestionDto, request: ClassificationRequest, model: String): ClassificationSuggestion {
        return ClassificationSuggestion(
            documentId = request.documentId,
            model = model,
            title = answer.title.cleaned(),
            category = categoryOf(name = answer.category.cleaned(), request = request),
            tags = tagsOf(names = answer.tags, request = request),
            documentDate = answer.documentDate.cleaned()?.let { dateOf(text = it) },
            correspondent = answer.correspondent.cleaned(),
            fileName = fileNameOf(name = answer.fileName.cleaned(), current = request.currentFileName),
            transcription = answer.transcription.cleaned(),
            confidence = answer.confidence?.coerceIn(range = 0.0..1.0),
        )
    }

    /** An existing category by name or path, ignoring case; otherwise a new one, below its parent if named. */
    private fun categoryOf(name: String?, request: ClassificationRequest): SuggestedCategory? {
        if (name == null) {
            return null
        }
        val key = name.lowercase(Locale.ROOT)
        val existing = request.categories.firstOrNull { category ->
            category.name.lowercase(Locale.ROOT) == key ||
                PromptTexts.pathOf(category = category, categories = request.categories).lowercase(Locale.ROOT) == key
        }
        if (existing != null) {
            return SuggestedCategory.Existing(categoryId = existing.id)
        }
        val parentName = name.substringBeforeLast(delimiter = "/", missingDelimiterValue = "").trim()
        val parent = request.categories.firstOrNull {
            parentName.isNotEmpty() &&
                PromptTexts.pathOf(category = it, categories = request.categories).equals(parentName, ignoreCase = true)
        }
        val ownName = name.substringAfterLast(delimiter = "/").trim()
        return ownName.takeIf { it.isNotEmpty() }?.let { SuggestedCategory.New(name = it, parentId = parent?.id) }
    }

    private fun tagsOf(names: List<String>, request: ClassificationRequest): List<SuggestedTag> {
        return names
            .mapNotNull { raw -> runCatching { TagName.of(raw = raw) }.getOrNull() }
            .distinct()
            .take(n = MAX_TAGS)
            .map { tag ->
                SuggestedTag(name = request.tags.firstOrNull { it == tag } ?: tag, isNew = tag !in request.tags)
            }
    }

    private fun dateOf(text: String): LocalDate? {
        return try {
            LocalDate.parse(text)
        } catch (ignored: DateTimeParseException) {
            null
        }
    }

    /** Keeps the file's extension, and drops a name the file name policy refuses. */
    private fun fileNameOf(name: String?, current: String): String? {
        if (name == null) {
            return null
        }
        val extension = current.substringAfterLast(delimiter = '.', missingDelimiterValue = "")
        val otherExtension = name.substringAfterLast(delimiter = '.', missingDelimiterValue = "")
        val withExtension = when {
            extension.isEmpty() || name.endsWith(suffix = ".$extension", ignoreCase = true) -> name
            EXTENSION.matches(input = otherExtension) -> "${name.substringBeforeLast(delimiter = '.')}.$extension"
            else -> "$name.$extension"
        }.trim()
        return withExtension.takeIf { FileNamePolicy.check(name = it) == null && it != current }
    }

    private fun String?.cleaned(): String? {
        return this?.trim()?.takeIf { it.isNotEmpty() && !it.equals(other = "null", ignoreCase = true) }
    }
}
