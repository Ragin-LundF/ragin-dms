package dms.ai.client.dto

import kotlinx.serialization.Serializable

/** The answer of `GET /models`. */
@Serializable
data class ModelsResponseDto(
    val data: List<ModelDto> = emptyList(),
)
