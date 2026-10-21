package dms.ai.client

import dms.ai.client.dto.ChatResponseDto
import dms.ai.client.dto.ModelsResponseDto
import dms.ai.client.model.ChatReply
import dms.application.port.ai.model.ClassificationFailure
import dms.application.port.ai.model.ModelList
import dms.application.port.state.model.AiSettings
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.plugins.timeout
import io.ktor.client.request.get
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.client.statement.HttpResponse
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.HttpStatusCode
import io.ktor.http.contentType
import io.ktor.http.isSuccess
import kotlinx.serialization.json.JsonObject
import java.io.IOException
import java.net.SocketTimeoutException

/**
 * The HTTP side of any server with the OpenAI chat completions API, LM Studio by default: the model
 * list, one chat request, and the mapping of errors. The settings are read for every call, so a
 * change in the settings applies to the next request.
 */
class ChatTransport(
    private val http: HttpClient,
    private val settings: suspend () -> AiSettings,
) {

    suspend fun models(settings: AiSettings): ModelList {
        return try {
            val response = http.get(urlString = "${baseOf(settings = settings)}/models") {
                timeout { requestTimeoutMillis = MODELS_TIMEOUT_MILLIS }
            }
            if (response.status.isSuccess()) {
                ModelList.Available(models = response.body<ModelsResponseDto>().data.map { it.id })
            } else {
                ModelList.Unavailable(reason = ClassificationFailure.PROVIDER_UNAVAILABLE, detail = detailOf(response))
            }
        } catch (exception: IOException) {
            ModelList.Unavailable(reason = failureOf(exception = exception), detail = exception.message)
        } catch (exception: IllegalArgumentException) {
            // An unresolvable host, an invalid URL, or an answer that is no model list.
            ModelList.Unavailable(reason = ClassificationFailure.PROVIDER_UNAVAILABLE, detail = exception.message)
        }
    }

    /**
     * Sends the request [body] builds for the chosen model. [hasImages] tells a refused request apart:
     * a model without vision answers images with HTTP 400. A [structured] request asks for JSON; its
     * answer may come as the reasoning of a reasoning model when the answer itself is empty.
     */
    suspend fun complete(hasImages: Boolean, structured: Boolean = false, body: (String) -> JsonObject): ChatReply {
        val ai = settings()
        val model = ai.model ?: when (val models = models(settings = ai)) {
            is ModelList.Unavailable -> return ChatReply.Failed(reason = models.reason, detail = models.detail)
            is ModelList.Available -> ModelChoice.of(models = models.models)
                ?: return ChatReply.Failed(
                    reason = ClassificationFailure.PROVIDER_UNAVAILABLE,
                    detail = "The AI server has no chat model",
                )
        }
        return try {
            val response = http.post(urlString = "${baseOf(settings = ai)}/chat/completions") {
                contentType(type = ContentType.Application.Json)
                setBody(body = body(model))
                timeout {
                    requestTimeoutMillis = ai.timeoutSeconds * MILLIS
                    socketTimeoutMillis = ai.timeoutSeconds * MILLIS
                }
            }
            replyOf(response = response, hasImages = hasImages, structured = structured, model = model)
        } catch (exception: IOException) {
            ChatReply.Failed(reason = failureOf(exception = exception), detail = exception.message)
        } catch (exception: IllegalArgumentException) {
            ChatReply.Failed(reason = ClassificationFailure.INVALID_RESPONSE, detail = exception.message)
        }
    }

    private suspend fun replyOf(
        response: HttpResponse,
        hasImages: Boolean,
        structured: Boolean,
        model: String,
    ): ChatReply {
        if (!response.status.isSuccess()) {
            val reason = if (hasImages && response.status == HttpStatusCode.BadRequest) {
                ClassificationFailure.MODEL_CANNOT_READ_IMAGES
            } else {
                ClassificationFailure.PROVIDER_UNAVAILABLE
            }
            return ChatReply.Failed(reason = reason, detail = detailOf(response))
        }
        val message = response.body<ChatResponseDto>().choices.firstOrNull()?.message
        val content = message?.content?.let { AnswerText.clean(content = it) }
            ?.takeIf { it.isNotEmpty() }
            ?: message?.reasoningContent?.takeIf { structured }?.let { AnswerText.clean(content = it) }
        return if (content.isNullOrEmpty()) {
            ChatReply.Failed(reason = ClassificationFailure.INVALID_RESPONSE, detail = "The AI answered nothing")
        } else {
            ChatReply.Answered(content = content, model = model)
        }
    }

    private suspend fun detailOf(response: HttpResponse): String {
        return "HTTP ${response.status.value}: ${response.bodyAsText().take(n = DETAIL_LENGTH)}"
    }

    private fun failureOf(exception: IOException): ClassificationFailure {
        val timedOut = exception is SocketTimeoutException ||
            exception::class.simpleName.orEmpty().contains(other = "Timeout")
        return if (timedOut) ClassificationFailure.TIMEOUT else ClassificationFailure.PROVIDER_UNAVAILABLE
    }

    private fun baseOf(settings: AiSettings): String {
        return settings.baseUrl.trim().trimEnd('/')
    }

    companion object {
        const val DETAIL_LENGTH: Int = 300
        private const val MILLIS = 1_000L
        private const val MODELS_TIMEOUT_MILLIS = 10_000L
    }
}
