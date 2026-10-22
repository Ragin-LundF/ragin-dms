package dms.storage.catalog.dto.settings

import com.fasterxml.jackson.annotation.JsonInclude
import com.fasterxml.jackson.annotation.JsonPropertyOrder
import dms.storage.catalog.dto.FieldValueDto

/** One field condition of a saved search, such as `amount GREATER_THAN 100`. */
@JsonInclude(JsonInclude.Include.NON_NULL)
@JsonPropertyOrder("key", "comparison", "value")
data class FieldConditionDto(
    val key: String,
    val comparison: String,
    val value: FieldValueDto? = null,
)
