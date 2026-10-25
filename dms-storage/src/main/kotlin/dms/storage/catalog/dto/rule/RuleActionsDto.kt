package dms.storage.catalog.dto.rule

import com.fasterxml.jackson.annotation.JsonAnyGetter
import com.fasterxml.jackson.annotation.JsonAnySetter
import com.fasterxml.jackson.annotation.JsonInclude
import com.fasterxml.jackson.annotation.JsonPropertyOrder
import dms.storage.catalog.dto.ExtensibleDto
import dms.storage.catalog.dto.FieldValueDto

@JsonInclude(JsonInclude.Include.NON_EMPTY)
@JsonPropertyOrder("categoryId", "tags", "fields", "targetFolder")
data class RuleActionsDto(
    val categoryId: String? = null,
    val tags: List<String> = emptyList(),
    val fields: Map<String, FieldValueDto> = emptyMap(),
    val targetFolder: String? = null,
) : ExtensibleDto {
    @get:JsonAnyGetter
    @field:JsonAnySetter
    override val unknownFields: MutableMap<String, Any?> = linkedMapOf()
}
