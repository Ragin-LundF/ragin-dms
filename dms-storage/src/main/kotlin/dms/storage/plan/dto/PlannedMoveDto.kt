package dms.storage.plan.dto

import com.fasterxml.jackson.annotation.JsonPropertyOrder

@JsonPropertyOrder("documentId", "from", "to")
data class PlannedMoveDto(
    val documentId: String,
    val from: String,
    val to: String,
)
