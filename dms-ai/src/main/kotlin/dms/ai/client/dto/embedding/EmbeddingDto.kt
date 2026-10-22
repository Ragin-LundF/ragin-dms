package dms.ai.client.dto.embedding

import kotlinx.serialization.Serializable

/** One vector of the answer of `POST /embeddings`, at [index] of the texts sent. */
@Serializable
data class EmbeddingDto(
    val index: Int = 0,
    val embedding: List<Float> = emptyList(),
)
