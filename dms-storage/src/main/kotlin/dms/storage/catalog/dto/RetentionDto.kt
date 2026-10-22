package dms.storage.catalog.dto

import com.fasterxml.jackson.annotation.JsonInclude
import com.fasterxml.jackson.annotation.JsonPropertyOrder

/** The opt-in retention of one document: a keep-until day and when the user marked it done. */
@JsonInclude(JsonInclude.Include.NON_NULL)
@JsonPropertyOrder("keepUntil", "settledAt")
data class RetentionDto(
    val keepUntil: String? = null,
    val settledAt: String? = null,
)
