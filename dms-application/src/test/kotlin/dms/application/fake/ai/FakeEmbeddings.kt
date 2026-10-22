package dms.application.fake.ai

import dms.application.port.ai.EmbeddingClient
import dms.application.port.ai.model.ClassificationFailure
import dms.application.port.ai.model.EmbeddingResult
import java.util.Locale

/**
 * Embeddings by counting the words of [VOCABULARY] in a text, so texts about the same things get
 * close vectors, deterministically. Fails while [available] is off.
 */
class FakeEmbeddings : EmbeddingClient {

    var available: Boolean = true
    var model: String = "fake-embed"
    var calls: Int = 0
        private set

    override suspend fun model(): String? {
        return model.takeIf { available }
    }

    override suspend fun embed(texts: List<String>): EmbeddingResult {
        calls++
        if (!available) {
            return EmbeddingResult.Failed(reason = ClassificationFailure.PROVIDER_UNAVAILABLE, detail = "offline")
        }
        return EmbeddingResult.Vectors(model = model, vectors = texts.map { vectorOf(text = it) })
    }

    private fun vectorOf(text: String): FloatArray {
        val lower = text.lowercase(Locale.ROOT)
        return FloatArray(size = VOCABULARY.size) { index -> Regex(
            pattern = VOCABULARY[index],
        ).findAll(input = lower).count().toFloat() }
    }

    companion object {
        val VOCABULARY: List<String> = listOf(
            "electricity",
            "power",
            "meter",
            "insurance",
            "car",
            "policy",
            "tax",
            "doctor",
        )
    }
}
