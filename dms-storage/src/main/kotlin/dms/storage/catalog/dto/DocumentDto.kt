package dms.storage.catalog.dto

import com.fasterxml.jackson.annotation.JsonAnyGetter
import com.fasterxml.jackson.annotation.JsonAnySetter
import com.fasterxml.jackson.annotation.JsonInclude
import com.fasterxml.jackson.annotation.JsonPropertyOrder

/** One document. Dates are ISO-8601 strings, so the file reads the same on every device. */
@JsonInclude(JsonInclude.Include.NON_EMPTY)
@JsonPropertyOrder(
    "id",
    "path",
    "contentHash",
    "size",
    "addedAt",
    "title",
    "documentDate",
    "correspondent",
    "categoryId",
    "tags",
    "fields",
    "note",
    "retention",
    "classification",
)
data class DocumentDto(
    val id: String,
    val path: String,
    val contentHash: String,
    val size: Long,
    val addedAt: String,
    val title: String? = null,
    val documentDate: String? = null,
    val correspondent: String? = null,
    val categoryId: String? = null,
    val tags: List<String> = emptyList(),
    val classification: ClassificationDto? = null,
    val fields: Map<String, FieldValueDto> = emptyMap(),
    val note: String? = null,
    val retention: RetentionDto? = null,
) : ExtensibleDto {
    @get:JsonAnyGetter
    @field:JsonAnySetter
    override val unknownFields: MutableMap<String, Any?> = linkedMapOf()
}
