package dms.ai.client.dto

import kotlinx.serialization.Serializable

/** The part of an OpenAI-compatible chat completion this tool reads. */
@Serializable
data class ChatResponseDto(
    val choices: List<ChoiceDto> = emptyList(),
)
