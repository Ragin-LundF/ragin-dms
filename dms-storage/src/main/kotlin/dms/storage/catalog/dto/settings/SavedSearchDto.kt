package dms.storage.catalog.dto.settings

import com.fasterxml.jackson.annotation.JsonAnyGetter
import com.fasterxml.jackson.annotation.JsonAnySetter
import com.fasterxml.jackson.annotation.JsonPropertyOrder
import dms.storage.catalog.dto.ExtensibleDto

@JsonPropertyOrder("id", "name", "query")
data class SavedSearchDto(
    val id: String,
    val name: String,
    val query: QueryDto = QueryDto(),
) : ExtensibleDto {
    @get:JsonAnyGetter
    @field:JsonAnySetter
    override val unknownFields: MutableMap<String, Any?> = linkedMapOf()
}
