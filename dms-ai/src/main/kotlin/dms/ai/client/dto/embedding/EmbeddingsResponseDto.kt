package dms.ai.client.dto.embedding

import kotlinx.serialization.Serializable

/** The answer of `POST /embeddings`. */
@Serializable
data class EmbeddingsResponseDto(
    val data: List<EmbeddingDto> = emptyList(),
    val model: String? = null,
)
