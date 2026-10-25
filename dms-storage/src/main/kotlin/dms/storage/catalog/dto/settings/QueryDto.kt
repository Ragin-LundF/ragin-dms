package dms.storage.catalog.dto.settings

import com.fasterxml.jackson.annotation.JsonAnyGetter
import com.fasterxml.jackson.annotation.JsonAnySetter
import com.fasterxml.jackson.annotation.JsonInclude
import com.fasterxml.jackson.annotation.JsonPropertyOrder
import dms.storage.catalog.dto.ExtensibleDto

/** A document query; only the conditions that are set are written. Dates are ISO-8601. */
@JsonInclude(JsonInclude.Include.NON_EMPTY)
@JsonPropertyOrder(
    "text",
    "category",
    "uncategorized",
    "tagsAll",
    "tagsAny",
    "tagsNone",
    "untagged",
    "from",
    "to",
    "correspondent",
    "folder",
    "fields",
    "addedSince",
    "contentText",
)
data class QueryDto(
    val text: String? = null,
    val category: String? = null,
    val uncategorized: Boolean? = null,
    val tagsAll: List<String> = emptyList(),
    val tagsAny: List<String> = emptyList(),
    val tagsNone: List<String> = emptyList(),
    val untagged: Boolean? = null,
    val from: String? = null,
    val to: String? = null,
    val correspondent: String? = null,
    val folder: String? = null,
    val fields: List<FieldConditionDto> = emptyList(),
    val addedSince: String? = null,
    val contentText: String? = null,
) : ExtensibleDto {
    @get:JsonAnyGetter
    @field:JsonAnySetter
    override val unknownFields: MutableMap<String, Any?> = linkedMapOf()
}
