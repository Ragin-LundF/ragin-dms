package dms.ai.client.dto

import kotlinx.serialization.Serializable

/** One custom field value the model read from the document, as plain text. */
@Serializable
data class FieldAnswerDto(
    val key: String = "",
    val value: String? = null,
)
