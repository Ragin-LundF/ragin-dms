package dms.storage.catalog.dto.rule

import com.fasterxml.jackson.annotation.JsonAnyGetter
import com.fasterxml.jackson.annotation.JsonAnySetter
import com.fasterxml.jackson.annotation.JsonPropertyOrder
import dms.storage.catalog.dto.ExtensibleDto

/** A matching rule; [origin] is `LEARNED` or `MANUAL`. */
@JsonPropertyOrder("id", "name", "origin", "enabled", "conditions", "actions")
data class RuleDto(
    val id: String,
    val name: String,
    val origin: String,
    val enabled: Boolean = true,
    val conditions: List<RuleConditionDto> = emptyList(),
    val actions: RuleActionsDto = RuleActionsDto(),
) : ExtensibleDto {
    @get:JsonAnyGetter
    @field:JsonAnySetter
    override val unknownFields: MutableMap<String, Any?> = linkedMapOf()
}
