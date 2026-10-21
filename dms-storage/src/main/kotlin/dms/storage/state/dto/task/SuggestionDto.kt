package dms.storage.state.dto.task

import com.fasterxml.jackson.annotation.JsonInclude

/** A pending AI suggestion. A suggested category is either [existingCategoryId] or [newCategoryName]. */
@JsonInclude(JsonInclude.Include.NON_EMPTY)
data class SuggestionDto(
    val documentId: String,
    val model: String,
    val title: String? = null,
    val existingCategoryId: String? = null,
    val newCategoryName: String? = null,
    val newCategoryParentId: String? = null,
    val tags: List<SuggestedTagDto> = emptyList(),
    val documentDate: String? = null,
    val correspondent: String? = null,
    val fileName: String? = null,
    val transcription: String? = null,
    val confidence: Double? = null,
)
