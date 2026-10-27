package dms.ai.client

import dms.ai.client.dto.ChatResponseDto
import dms.ai.client.dto.MessagesResponseDto
import dms.ai.client.dto.ModelsResponseDto
import dms.ai.client.model.ChatReply
import dms.application.port.ai.model.ClassificationFailure
import dms.application.port.ai.model.ModelList
import dms.application.port.state.model.AiProvider
import dms.application.port.state.model.AiSettingsModel
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.plugins.timeout
import io.ktor.client.request.HttpRequestBuilder
import io.ktor.client.request.get
import io.ktor.client.request.header
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.client.statement.HttpResponse
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.HttpStatusCode
import io.ktor.http.contentType
import io.ktor.http.isSuccess
import io.ktor.serialization.ContentConvertException
import kotlinx.serialization.json.JsonObject
import java.io.IOException
import java.net.SocketTimeoutException

/**
 * The HTTP side of any server with the OpenAI chat completions API, LM Studio by default, or of
 * Anthropic's Messages API when the settings choose [AiProvider.ANTHROPIC]: the model list, one chat request,
 * and the mapping of errors. The settings are read for every call, so a change in the settings applies
 * to the next request. [anthropicKey] is the Anthropic API key, read from the environment by default.
 */
class ChatTransport(
    private val http: HttpClient,
    private val settings: suspend () -> AiSettingsModel,
    private val anthropicKey: () -> String?,
) {

    suspend fun models(settings: AiSettingsModel): ModelList {
        missingKey(settings = settings)?.let { detail ->
            return ModelList.Unavailable(reason = ClassificationFailure.PROVIDER_UNAVAILABLE, detail = detail)
        }
        return try {
            val response = http.get(urlString = "${baseOf(settings = settings)}/models") {
                authorize(settings = settings)
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
            // An unresolvable host or an invalid URL.
            ModelList.Unavailable(reason = ClassificationFailure.PROVIDER_UNAVAILABLE, detail = exception.message)
        } catch (exception: ContentConvertException) {
            // An answer that is no model list; Ktor wraps every deserialization error in this.
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
        missingKey(settings = ai)?.let { detail ->
            return ChatReply.Failed(reason = ClassificationFailure.PROVIDER_UNAVAILABLE, detail = detail)
        }
        val anthropic = isAnthropic(settings = ai)
        val chosen = if (anthropic) ai.anthropicModel.takeIf { it.isNotBlank() } else ai.model
        val model = chosen ?: when (val models = models(settings = ai)) {
            is ModelList.Unavailable -> return ChatReply.Failed(reason = models.reason, detail = models.detail)
            is ModelList.Available -> ModelChoice.of(models = models.models)
                ?: return ChatReply.Failed(
                    reason = ClassificationFailure.PROVIDER_UNAVAILABLE,
                    detail = "The AI server has no chat model",
                )
        }
        return try {
            val path = if (anthropic) "messages" else "chat/completions"
            val response = http.post(urlString = "${baseOf(settings = ai)}/$path") {
                authorize(settings = ai)
                contentType(type = ContentType.Application.Json)
                setBody(body = if (anthropic) AnthropicRequest.of(openAi = body(model)) else body(model))
                timeout {
                    requestTimeoutMillis = ai.timeoutSeconds * MILLIS
                    socketTimeoutMillis = ai.timeoutSeconds * MILLIS
                }
            }
            replyOf(
                response = response,
                hasImages = hasImages,
                structured = structured,
                model = model,
                anthropic = anthropic,
            )
        } catch (exception: IOException) {
            ChatReply.Failed(reason = failureOf(exception = exception), detail = exception.message)
        } catch (exception: IllegalArgumentException) {
            ChatReply.Failed(reason = ClassificationFailure.INVALID_RESPONSE, detail = exception.message)
        } catch (exception: ContentConvertException) {
            // An answer that is no JSON or not the chat schema; Ktor wraps every deserialization error in this.
            ChatReply.Failed(reason = ClassificationFailure.INVALID_RESPONSE, detail = exception.message)
        }
    }

    private suspend fun replyOf(
        response: HttpResponse,
        hasImages: Boolean,
        structured: Boolean,
        model: String,
        anthropic: Boolean,
    ): ChatReply {
        if (!response.status.isSuccess()) {
            // Anthropic answers 400 for many reasons (schema, image size), not for a model without vision.
            val reason = if (hasImages && !anthropic && response.status == HttpStatusCode.BadRequest) {
                ClassificationFailure.MODEL_CANNOT_READ_IMAGES
            } else {
                ClassificationFailure.PROVIDER_UNAVAILABLE
            }
            return ChatReply.Failed(reason = reason, detail = detailOf(response))
        }
        val content = if (anthropic) {
            val message = response.body<MessagesResponseDto>()
            if (message.stopReason == REFUSAL) {
                return ChatReply.Failed(reason = ClassificationFailure.INVALID_RESPONSE, detail = "The AI declined")
            }
            val text = message.content.filter { it.type == TEXT_BLOCK }
                .joinToString(separator = "") { it.text.orEmpty() }
            AnswerText.clean(content = text)
        } else {
            openAiContentOf(response = response, structured = structured)
        }
        return if (content.isNullOrEmpty()) {
            ChatReply.Failed(reason = ClassificationFailure.INVALID_RESPONSE, detail = "The AI answered nothing")
        } else {
            ChatReply.Answered(content = content, model = model)
        }
    }

    private suspend fun openAiContentOf(response: HttpResponse, structured: Boolean): String? {
        val message = response.body<ChatResponseDto>().choices.firstOrNull()?.message
        return message?.content?.let { AnswerText.clean(content = it) }
            ?.takeIf { it.isNotEmpty() }
            ?: message?.reasoningContent?.takeIf { structured }?.let { AnswerText.clean(content = it) }
    }

    private fun isAnthropic(settings: AiSettingsModel): Boolean {
        return settings.provider == AiProvider.ANTHROPIC
    }

    private fun missingKey(settings: AiSettingsModel): String? {
        val missing = isAnthropic(settings = settings) && anthropicKey().isNullOrBlank()
        return if (missing) "$ANTHROPIC_KEY_VARIABLE is not set" else null
    }

    private fun HttpRequestBuilder.authorize(settings: AiSettingsModel) {
        if (isAnthropic(settings = settings)) {
            header(key = "x-api-key", value = anthropicKey())
            header(key = "anthropic-version", value = ANTHROPIC_VERSION)
        }
    }

    private suspend fun detailOf(response: HttpResponse): String {
        return "HTTP ${response.status.value}: ${response.bodyAsText().take(n = DETAIL_LENGTH)}"
    }

    /** A timeout, or an unreachable server for every other I/O failure. */
    internal fun failureOf(exception: IOException): ClassificationFailure {
        val timedOut = exception is SocketTimeoutException ||
            exception::class.simpleName.orEmpty().contains(other = "Timeout")
        return if (timedOut) ClassificationFailure.TIMEOUT else ClassificationFailure.PROVIDER_UNAVAILABLE
    }

    private fun baseOf(settings: AiSettingsModel): String {
        return if (isAnthropic(settings = settings)) ANTHROPIC_BASE_URL else settings.baseUrl.trim().trimEnd('/')
    }

    companion object {
        const val DETAIL_LENGTH: Int = 300
        private const val MILLIS = 1_000L
        private const val MODELS_TIMEOUT_MILLIS = 10_000L
        private const val ANTHROPIC_BASE_URL = "https://api.anthropic.com/v1"
        private const val ANTHROPIC_VERSION = "2023-06-01"
        private const val REFUSAL = "refusal"
        private const val TEXT_BLOCK = "text"
        const val ANTHROPIC_KEY_VARIABLE: String = "ANTHROPIC_API_KEY"
    }
}
