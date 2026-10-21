package dms.ai.client

import dms.ai.client.model.ChatReply
import dms.ai.prompt.SuggestionMapper
import dms.application.port.ai.AiServer
import dms.application.port.ai.DocumentClassifier
import dms.application.port.ai.model.ClassificationFailure
import dms.application.port.ai.model.ClassificationRequest
import dms.application.port.ai.model.ClassificationResult
import dms.application.port.ai.model.DocumentContent
import dms.application.port.ai.model.ModelList
import dms.application.port.state.model.AiSettings

/** Classifies documents and lists the models of any server with the OpenAI chat completions API. */
class OpenAiCompatibleClient(private val transport: ChatTransport) : DocumentClassifier, AiServer {

    override suspend fun models(settings: AiSettings): ModelList {
        return transport.models(settings = settings)
    }

    override suspend fun suggest(request: ClassificationRequest): ClassificationResult {
        val images = request.content is DocumentContent.PageImages
        val reply = transport.complete(hasImages = images, structured = true) { model ->
            ChatRequestBuilder.build(model = model, request = request)
        }
        return when (reply) {
            is ChatReply.Failed -> ClassificationResult.Failed(reason = reply.reason, detail = reply.detail)
            is ChatReply.Answered -> {
                val answer = ResponseParser.parse(content = reply.content)
                    ?: return ClassificationResult.Failed(
                        reason = ClassificationFailure.INVALID_RESPONSE,
                        detail = reply.content.take(n = ChatTransport.DETAIL_LENGTH),
                    )
                ClassificationResult.Suggested(
                    suggestion = SuggestionMapper.toSuggestion(answer = answer, request = request, model = reply.model),
                )
            }
        }
    }
}
