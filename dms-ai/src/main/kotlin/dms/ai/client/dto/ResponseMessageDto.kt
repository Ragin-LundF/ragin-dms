package dms.ai.client.dto

import kotlinx.serialization.Serializable

@Serializable
data class ResponseMessageDto(
    val content: String? = null,
)
