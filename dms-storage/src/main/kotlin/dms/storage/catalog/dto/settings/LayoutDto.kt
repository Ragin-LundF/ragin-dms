package dms.storage.catalog.dto.settings

import com.fasterxml.jackson.annotation.JsonAnyGetter
import com.fasterxml.jackson.annotation.JsonAnySetter
import com.fasterxml.jackson.annotation.JsonInclude
import com.fasterxml.jackson.annotation.JsonPropertyOrder
import dms.storage.catalog.dto.ExtensibleDto

@JsonInclude(JsonInclude.Include.NON_NULL)
@JsonPropertyOrder("inbox", "pathTemplate", "defaultCurrency", "embedMetadata")
data class LayoutDto(
    val inbox: InboxDto? = null,
    val pathTemplate: String? = null,
    val defaultCurrency: String? = null,
    val embedMetadata: Boolean? = null,
) : ExtensibleDto {
    @get:JsonAnyGetter
    @field:JsonAnySetter
    override val unknownFields: MutableMap<String, Any?> = linkedMapOf()
}
