package dms.storage.catalog.dto

import com.fasterxml.jackson.annotation.JsonAnyGetter
import com.fasterxml.jackson.annotation.JsonAnySetter
import com.fasterxml.jackson.annotation.JsonInclude
import com.fasterxml.jackson.annotation.JsonPropertyOrder

@JsonInclude(JsonInclude.Include.NON_EMPTY)
@JsonPropertyOrder("id", "name", "parentId", "retentionYears")
data class CategoryDto(
    val id: String,
    val name: String,
    val parentId: String? = null,
    val retentionYears: Int? = null,
) {
    @get:JsonAnyGetter
    @field:JsonAnySetter
    val unknownFields: MutableMap<String, Any?> = linkedMapOf()
}
