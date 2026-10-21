package dms.ai.client

import dms.ai.client.dto.ChatResponseDto
import dms.ai.client.dto.ModelsResponseDto
import dms.ai.prompt.SuggestionMapper
import dms.application.port.ai.AiServer
import dms.application.port.ai.DocumentClassifier
import dms.application.port.ai.model.ClassificationFailure
import dms.application.port.ai.model.ClassificationRequest
import dms.application.port.ai.model.ClassificationResult
import dms.application.port.ai.model.DocumentContent
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
import java.io.IOException
import java.net.SocketTimeoutException

/**
 * Talks to any server with the OpenAI chat completions API, LM Studio by default. The settings are
 * read for every call, so a change in the settings applies to the next document.
 */
class OpenAiCompatibleClient(
    private val http: HttpClient,
    private val settings: suspend () -> AiSettings,
) : DocumentClassifier, AiServer {

    override suspend fun models(settings: AiSettings): ModelList {
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

    override suspend fun suggest(request: ClassificationRequest): ClassificationResult {
        val ai = settings()
        val model = ai.model ?: when (val models = models(settings = ai)) {
            is ModelList.Unavailable -> return ClassificationResult.Failed(
                reason = models.reason,
                detail = models.detail,
            )

            is ModelList.Available -> models.models.firstOrNull()
                ?: return ClassificationResult.Failed(
                    reason = ClassificationFailure.PROVIDER_UNAVAILABLE,
                    detail = "The AI server has no model loaded",
                )
        }
        return try {
            val response = http.post(urlString = "${baseOf(settings = ai)}/chat/completions") {
                contentType(type = ContentType.Application.Json)
                setBody(body = ChatRequestBuilder.build(model = model, request = request))
                timeout {
                    requestTimeoutMillis = ai.timeoutSeconds * MILLIS
                    socketTimeoutMillis = ai.timeoutSeconds * MILLIS
                }
            }
            resultOf(response = response, request = request, model = model)
        } catch (exception: IOException) {
            ClassificationResult.Failed(reason = failureOf(exception = exception), detail = exception.message)
        } catch (exception: IllegalArgumentException) {
            ClassificationResult.Failed(reason = ClassificationFailure.INVALID_RESPONSE, detail = exception.message)
        }
    }

    private suspend fun resultOf(
        response: HttpResponse,
        request: ClassificationRequest,
        model: String,
    ): ClassificationResult {
        if (!response.status.isSuccess()) {
            val images = request.content is DocumentContent.PageImages
            val reason = if (images && response.status == HttpStatusCode.BadRequest) {
                ClassificationFailure.MODEL_CANNOT_READ_IMAGES
            } else {
                ClassificationFailure.PROVIDER_UNAVAILABLE
            }
            return ClassificationResult.Failed(reason = reason, detail = detailOf(response))
        }
        val content = response.body<ChatResponseDto>().choices.firstOrNull()?.message?.content
        val answer = content?.let { ResponseParser.parse(content = it) }
            ?: return ClassificationResult.Failed(
                reason = ClassificationFailure.INVALID_RESPONSE,
                detail = content?.take(n = DETAIL_LENGTH),
            )
        return ClassificationResult.Suggested(
            suggestion = SuggestionMapper.toSuggestion(answer = answer, request = request, model = model),
        )
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

    private companion object {
        const val MILLIS = 1_000L
        const val MODELS_TIMEOUT_MILLIS = 10_000L
        const val DETAIL_LENGTH = 300
    }
}
