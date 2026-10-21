package dms.ai.client.dto

import kotlinx.serialization.Serializable

@Serializable
data class ChoiceDto(
    val message: ResponseMessageDto? = null,
)
