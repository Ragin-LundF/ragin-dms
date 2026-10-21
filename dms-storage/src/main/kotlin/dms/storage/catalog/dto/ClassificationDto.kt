package dms.storage.catalog.dto

import com.fasterxml.jackson.annotation.JsonInclude
import com.fasterxml.jackson.annotation.JsonPropertyOrder

@JsonInclude(JsonInclude.Include.NON_EMPTY)
@JsonPropertyOrder("source", "model", "confirmedAt")
data class ClassificationDto(
    val source: String,
    val model: String? = null,
    val confirmedAt: String,
)
