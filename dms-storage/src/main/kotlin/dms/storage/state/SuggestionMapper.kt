package dms.storage.state

import dms.domain.classification.ClassificationSuggestion
import dms.domain.classification.SuggestedCategory
import dms.domain.classification.SuggestedTag
import dms.domain.document.DocumentId
import dms.domain.taxonomy.CategoryId
import dms.domain.taxonomy.TagName
import dms.storage.state.dto.task.SuggestedTagDto
import dms.storage.state.dto.task.SuggestionDto
import java.time.LocalDate

/** Maps pending suggestions to and from `tasks.yaml`. */
object SuggestionMapper {

    fun toDto(suggestion: ClassificationSuggestion): SuggestionDto {
        val category = suggestion.category
        return SuggestionDto(
            documentId = suggestion.documentId.value,
            model = suggestion.model,
            title = suggestion.title,
            existingCategoryId = (category as? SuggestedCategory.Existing)?.categoryId?.value,
            newCategoryName = (category as? SuggestedCategory.New)?.name,
            newCategoryParentId = (category as? SuggestedCategory.New)?.parentId?.value,
            tags = suggestion.tags.map { SuggestedTagDto(name = it.name.value, isNew = it.isNew) },
            documentDate = suggestion.documentDate?.toString(),
            correspondent = suggestion.correspondent,
            fileName = suggestion.fileName,
            transcription = suggestion.transcription,
            confidence = suggestion.confidence,
        )
    }

    fun toDomain(dto: SuggestionDto): ClassificationSuggestion {
        return ClassificationSuggestion(
            documentId = DocumentId(value = dto.documentId),
            model = dto.model,
            title = dto.title,
            category = categoryOf(dto = dto),
            tags = dto.tags.map { SuggestedTag(name = TagName.of(raw = it.name), isNew = it.isNew) },
            documentDate = dto.documentDate?.let { LocalDate.parse(it) },
            correspondent = dto.correspondent,
            fileName = dto.fileName,
            transcription = dto.transcription,
            confidence = dto.confidence,
        )
    }

    private fun categoryOf(dto: SuggestionDto): SuggestedCategory? {
        if (dto.existingCategoryId != null) {
            return SuggestedCategory.Existing(categoryId = CategoryId(value = dto.existingCategoryId))
        }
        if (dto.newCategoryName != null) {
            return SuggestedCategory.New(
                name = dto.newCategoryName,
                parentId = dto.newCategoryParentId?.let { CategoryId(value = it) },
            )
        }
        return null
    }
}
