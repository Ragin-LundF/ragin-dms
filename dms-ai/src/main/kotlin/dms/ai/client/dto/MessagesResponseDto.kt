package dms.ai.client.dto

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/** The part of an Anthropic Messages API answer this tool reads. */
@Serializable
data class MessagesResponseDto(
    val content: List<ContentBlockDto> = emptyList(),
    @SerialName(value = "stop_reason")
    val stopReason: String? = null,
)
