package dms.storage.catalog.dto.settings

import com.fasterxml.jackson.annotation.JsonAnyGetter
import com.fasterxml.jackson.annotation.JsonAnySetter
import com.fasterxml.jackson.annotation.JsonInclude
import com.fasterxml.jackson.annotation.JsonPropertyOrder
import dms.storage.catalog.dto.ExtensibleDto

@JsonInclude(JsonInclude.Include.NON_EMPTY)
@JsonPropertyOrder("name", "aliases")
data class CorrespondentDto(
    val name: String,
    val aliases: List<String> = emptyList(),
) : ExtensibleDto {
    @get:JsonAnyGetter
    @field:JsonAnySetter
    override val unknownFields: MutableMap<String, Any?> = linkedMapOf()
}
