package dms.ai.client

import dms.ai.client.dto.embedding.EmbeddingsResponseDto
import dms.application.port.ai.EmbeddingClient
import dms.application.port.ai.model.ClassificationFailure
import dms.application.port.ai.model.EmbeddingResult
import dms.application.port.ai.model.ModelList
import dms.application.port.state.model.AiSettingsModel
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.plugins.timeout
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.contentType
import io.ktor.http.isSuccess
import kotlinx.serialization.json.add
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonArray
import java.io.IOException
import java.util.Locale

/**
 * Embeddings on any server with the OpenAI `/embeddings` API (LM Studio, Ollama's `/v1`). The model
 * is the one chosen in the AI settings, or else the first embedding model the server lists.
 */
class OpenAiEmbeddings(
    private val http: HttpClient,
    private val transport: ChatTransport,
    private val settings: suspend () -> AiSettingsModel,
) : EmbeddingClient {

    override suspend fun model(): String? {
        val ai = settings()
        ai.embeddingModel?.takeIf { it.isNotBlank() }?.let { return it }
        val models = transport.models(settings = ai) as? ModelList.Available ?: return null
        return models.models.firstOrNull { it.lowercase(Locale.ROOT).contains(other = "embed") }
    }

    override suspend fun embed(texts: List<String>): EmbeddingResult {
        val ai = settings()
        val model = model() ?: return EmbeddingResult.Failed(
            reason = ClassificationFailure.PROVIDER_UNAVAILABLE,
            detail = "No embedding model: load one in LM Studio or choose it in the AI settings",
        )
        return try {
            val response = http.post(urlString = "${ai.baseUrl.trim().trimEnd('/')}/embeddings") {
                contentType(type = ContentType.Application.Json)
                setBody(
                    body = buildJsonObject {
                        put(key = "model", value = model)
                        putJsonArray(key = "input") { texts.forEach { add(value = it) } }
                    },
                )
                timeout { requestTimeoutMillis = ai.timeoutSeconds * MILLIS }
            }
            if (!response.status.isSuccess()) {
                return EmbeddingResult.Failed(
                    reason = ClassificationFailure.PROVIDER_UNAVAILABLE,
                    detail = "HTTP ${response.status.value}: " +
                        response.bodyAsText().take(n = ChatTransport.DETAIL_LENGTH),
                )
            }
            val vectors = response.body<EmbeddingsResponseDto>().data
                .sortedBy { it.index }
                .map { it.embedding.toFloatArray() }
            if (vectors.size != texts.size || vectors.any { it.isEmpty() }) {
                EmbeddingResult.Failed(
                    reason = ClassificationFailure.INVALID_RESPONSE,
                    detail = "The server returned ${vectors.size} vectors",
                )
            } else {
                EmbeddingResult.Vectors(model = model, vectors = vectors)
            }
        } catch (exception: IOException) {
            EmbeddingResult.Failed(reason = ClassificationFailure.PROVIDER_UNAVAILABLE, detail = exception.message)
        } catch (exception: IllegalArgumentException) {
            EmbeddingResult.Failed(reason = ClassificationFailure.INVALID_RESPONSE, detail = exception.message)
        }
    }

    private companion object {
        const val MILLIS = 1_000L
    }
}
