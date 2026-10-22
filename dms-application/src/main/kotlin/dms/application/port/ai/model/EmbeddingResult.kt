package dms.application.port.ai.model

/** The embeddings of some texts, one vector per text in their order, from [model]; or why there are none. */
sealed interface EmbeddingResult {

    class Vectors(val model: String, val vectors: List<FloatArray>) : EmbeddingResult

    data class Failed(val reason: ClassificationFailure, val detail: String? = null) : EmbeddingResult
}
