package dms.storage.catalog.dto

import com.fasterxml.jackson.annotation.JsonInclude
import com.fasterxml.jackson.annotation.JsonPropertyOrder

/**
 * One custom field value. Exactly one of the value properties is set, which also tells its type:
 * `text`, `number`, `amount` with `currency`, `date` (ISO-8601) or `flag`.
 */
@JsonInclude(JsonInclude.Include.NON_NULL)
@JsonPropertyOrder("text", "number", "amount", "currency", "date", "flag")
data class FieldValueDto(
    val text: String? = null,
    val number: String? = null,
    val amount: String? = null,
    val currency: String? = null,
    val date: String? = null,
    val flag: Boolean? = null,
)
