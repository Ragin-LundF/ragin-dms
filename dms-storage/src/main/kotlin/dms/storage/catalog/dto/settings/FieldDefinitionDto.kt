package dms.storage.catalog.dto.settings

import com.fasterxml.jackson.annotation.JsonAnyGetter
import com.fasterxml.jackson.annotation.JsonAnySetter
import com.fasterxml.jackson.annotation.JsonInclude
import com.fasterxml.jackson.annotation.JsonPropertyOrder
import dms.storage.catalog.dto.ExtensibleDto

/** A custom field: key, label, type (`TEXT`, `NUMBER`, `MONEY`, `DATE`, `BOOLEAN`) and deadline role. */
@JsonInclude(JsonInclude.Include.NON_NULL)
@JsonPropertyOrder("key", "label", "type", "role")
data class FieldDefinitionDto(
    val key: String,
    val label: String,
    val type: String,
    val role: String? = null,
) : ExtensibleDto {
    @get:JsonAnyGetter
    @field:JsonAnySetter
    override val unknownFields: MutableMap<String, Any?> = linkedMapOf()
}
