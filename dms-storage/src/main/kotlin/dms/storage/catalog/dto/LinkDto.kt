package dms.storage.catalog.dto

import com.fasterxml.jackson.annotation.JsonAnyGetter
import com.fasterxml.jackson.annotation.JsonAnySetter
import com.fasterxml.jackson.annotation.JsonInclude
import com.fasterxml.jackson.annotation.JsonPropertyOrder

/** A link between two documents, by id; [first] is the smaller id. */
@JsonInclude(JsonInclude.Include.NON_NULL)
@JsonPropertyOrder("first", "second", "label")
data class LinkDto(
    val first: String,
    val second: String,
    val label: String? = null,
) : ExtensibleDto {
    @get:JsonAnyGetter
    @field:JsonAnySetter
    override val unknownFields: MutableMap<String, Any?> = linkedMapOf()
}
