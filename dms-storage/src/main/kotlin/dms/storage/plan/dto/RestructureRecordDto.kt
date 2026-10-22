package dms.storage.plan.dto

import com.fasterxml.jackson.annotation.JsonPropertyOrder

/** `.dms-plans/<name>.yaml`: an applied restructure, readable and reversible. */
@JsonPropertyOrder("appliedAt", "template", "moves")
data class RestructureRecordDto(
    val appliedAt: String,
    val template: String,
    val moves: List<PlannedMoveDto> = emptyList(),
)
