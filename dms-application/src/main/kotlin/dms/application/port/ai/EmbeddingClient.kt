package dms.application.port.ai

import dms.application.port.ai.model.EmbeddingResult

/** Turns texts into embedding vectors on the AI server (`/v1/embeddings` of LM Studio or Ollama). */
interface EmbeddingClient {

    /** The embedding model that [embed] uses now, or `null` if none is set and the server offers none. */
    suspend fun model(): String?

    suspend fun embed(texts: List<String>): EmbeddingResult
}
