package dms.ai.prompt

import dms.ai.client.dto.FieldAnswerDto
import dms.ai.client.dto.SuggestionDto
import dms.application.port.ai.model.ClassificationRequestModel
import dms.domain.classification.SuggestedCategory
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.classification.model.SuggestedFolderModel
import dms.domain.classification.model.SuggestedTagModel
import dms.domain.field.FieldValue
import dms.domain.field.FieldValues
import dms.domain.field.model.FieldKeyModel
import dms.domain.library.FileNamePolicy
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.TagNameModel
import java.time.LocalDate
import java.time.format.DateTimeParseException
import java.util.Locale

/**
 * Turns the model's answer into a suggestion the domain accepts. A value that does not fit is dropped
 * rather than failing the whole suggestion: an unknown date format, an invalid file name, a blank tag.
 */
object SuggestionMapper {

    private const val MAX_TAGS = 8
    private const val PAGE_BREAK = "\u000C"
    private val EXTENSION = Regex(pattern = "[A-Za-z0-9]{1,5}")

    fun toSuggestion(
        answer: SuggestionDto,
        request: ClassificationRequestModel,
        model: String,
    ): ClassificationSuggestionModel {
        return ClassificationSuggestionModel(
            documentId = request.documentId,
            model = model,
            title = answer.title.cleaned(),
            category = categoryOf(name = answer.category.cleaned(), request = request),
            tags = tagsOf(names = answer.tags, request = request),
            documentDate = answer.documentDate.cleaned()?.let { dateOf(text = it) },
            correspondent = answer.correspondent.cleaned(),
            fileName = fileNameOf(name = answer.fileName.cleaned(), current = request.currentFileName),
            transcription = transcriptionOf(answer = answer),
            confidence = answer.confidence?.coerceIn(range = 0.0..1.0),
            targetFolder = targetFolderOf(answer = answer, request = request),
            fields = fieldsOf(answers = answer.fields, request = request),
            targetReason = answer.targetReason.cleaned()?.lines()?.first(),
        )
    }

    /**
     * The text read on the page images: the pages separated by a form feed, so a text layer can put
     * each on its own page, or the whole transcription when the model gave no pages.
     */
    private fun transcriptionOf(answer: SuggestionDto): String? {
        val pages = answer.pages.map { it.trim() }
        if (pages.any { it.isNotEmpty() }) {
            return pages.joinToString(separator = PAGE_BREAK)
        }
        return answer.transcription.cleaned()
    }

    /** The values of defined fields that fit their type; anything else is dropped, never guessed. */
    private fun fieldsOf(
        answers: List<FieldAnswerDto>,
        request: ClassificationRequestModel,
    ): Map<FieldKeyModel, FieldValue> {
        val definitions = request.fieldDefinitions.associateBy { it.key.value }
        return answers.mapNotNull { answer ->
            val definition = definitions[answer.key.trim()] ?: return@mapNotNull null
            val raw = answer.value.cleaned() ?: return@mapNotNull null
            FieldValues.parse(type = definition.type, raw = raw, defaultCurrency = request.defaultCurrency)
                ?.let { definition.key to it }
        }.toMap()
    }

    /**
     * A folder of the outline, or a new folder directly below one that the model flagged as new; any
     * other path, and every path when no outline was sent, is dropped.
     */
    private fun targetFolderOf(answer: SuggestionDto, request: ClassificationRequestModel): SuggestedFolderModel? {
        val outline = request.outline ?: return null
        val raw = answer.targetFolder.cleaned() ?: return null
        val segments = raw.replace(oldChar = '\\', newChar = '/').split('/')
            .map { it.trim() }
            .filter { it.isNotEmpty() }
        if (segments.isEmpty() || segments.any { FileNamePolicy.check(name = it) != null }) {
            return null
        }
        val path = RelativePathModel.fromSegments(segments = segments)
        return when {
            outline.contains(path = path) -> SuggestedFolderModel(path = path, isNew = false)
            answer.targetFolderIsNew == true && outline.canCreate(path = path) -> SuggestedFolderModel(
                path = path,
                isNew = true,
            )
            else -> null
        }
    }

    /** An existing category by name or path, ignoring case; otherwise a new one, below its parent if named. */
    private fun categoryOf(name: String?, request: ClassificationRequestModel): SuggestedCategory? {
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

    private fun tagsOf(names: List<String>, request: ClassificationRequestModel): List<SuggestedTagModel> {
        return names
            .mapNotNull { raw -> runCatching { TagNameModel.of(raw = raw) }.getOrNull() }
            .distinct()
            .take(n = MAX_TAGS)
            .map { tag ->
                SuggestedTagModel(name = request.tags.firstOrNull { it == tag } ?: tag, isNew = tag !in request.tags)
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
