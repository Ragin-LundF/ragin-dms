package dms.ai.assistant

import dms.ai.client.AnswerText
import dms.ai.client.ChatRequestBuilder
import dms.ai.client.ChatTransport
import dms.ai.client.model.ChatMessageModel
import dms.ai.client.model.ChatReply
import dms.application.port.ai.model.ClassificationFailure
import dms.application.port.ai.model.DocumentContent
import dms.application.port.assistant.DocumentAssistant
import dms.application.port.assistant.model.AssistantAnswer
import dms.application.port.assistant.model.AssistantRequestModel
import dms.application.port.assistant.model.ChatRole
import dms.application.port.assistant.model.ChatTurnModel
import dms.domain.taxonomy.model.TagNameModel
import kotlinx.serialization.json.JsonPrimitive

/** The document assistant on any server with the OpenAI chat completions API. */
class OpenAiAssistant(private val transport: ChatTransport) : DocumentAssistant {

    override suspend fun summarize(request: AssistantRequestModel): AssistantAnswer {
        val messages = listOf(
            system(text = AssistantPrompts.summary()),
            user(text = AssistantPrompts.summaryRequest(request = request), request = request),
        )
        return textOf(reply = send(request = request, temperature = SUMMARY_TEMPERATURE, messages = messages))
    }

    override suspend fun suggestTags(request: AssistantRequestModel, existingTags: Set<TagNameModel>): AssistantAnswer {
        val messages = listOf(
            system(text = AssistantPrompts.tags(existingTags = existingTags)),
            user(text = AssistantPrompts.tagsRequest(request = request), request = request),
        )
        return when (val reply = send(request = request, temperature = TAGS_TEMPERATURE, messages = messages)) {
            is ChatReply.Failed -> AssistantAnswer.Failed(reason = reply.reason, detail = reply.detail)
            is ChatReply.Answered -> {
                val tags = TagAnswerParser.parse(answer = reply.content, existingTags = existingTags)
                if (tags.isEmpty()) {
                    AssistantAnswer.Failed(
                        reason = ClassificationFailure.INVALID_RESPONSE,
                        detail = reply.content.take(n = ChatTransport.DETAIL_LENGTH),
                    )
                } else {
                    AssistantAnswer.Tags(tags = tags, model = reply.model)
                }
            }
        }
    }

    override suspend fun answer(request: AssistantRequestModel, conversation: List<ChatTurnModel>): AssistantAnswer {
        val firstQuestion = conversation.indexOfFirst { it.role == ChatRole.USER }
        val turns = conversation.mapIndexed { index, turn ->
            if (index == firstQuestion) {
                user(text = turn.text, request = request)
            } else {
                ChatMessageModel(role = roleOf(turn = turn), content = JsonPrimitive(value = turn.text))
            }
        }
        val messages = listOf(system(text = AssistantPrompts.chat(request = request))) + turns
        return textOf(reply = send(request = request, temperature = CHAT_TEMPERATURE, messages = messages))
    }

    private suspend fun send(
        request: AssistantRequestModel,
        temperature: Double,
        messages: List<ChatMessageModel>,
    ): ChatReply {
        return transport.complete(hasImages = request.content is DocumentContent.PageImages) { model ->
            ChatRequestBuilder.chat(model = model, temperature = temperature, messages = messages)
        }
    }

    private fun textOf(reply: ChatReply): AssistantAnswer {
        return when (reply) {
            is ChatReply.Answered -> AssistantAnswer.Text(
                text = AnswerText.plain(content = reply.content),
                model = reply.model,
            )

            is ChatReply.Failed -> AssistantAnswer.Failed(reason = reply.reason, detail = reply.detail)
        }
    }

    private fun roleOf(turn: ChatTurnModel): String {
        return if (turn.role == ChatRole.ASSISTANT) "assistant" else "user"
    }

    private fun system(text: String): ChatMessageModel {
        return ChatMessageModel(role = "system", content = JsonPrimitive(value = text))
    }

    /** A user message; the document's page images go with it when the document was read as images. */
    private fun user(text: String, request: AssistantRequestModel): ChatMessageModel {
        val images = (request.content as? DocumentContent.PageImages)?.pages.orEmpty()
        return ChatMessageModel(role = "user", content = ChatRequestBuilder.content(text = text, images = images))
    }

    private companion object {
        const val SUMMARY_TEMPERATURE = 0.2
        const val TAGS_TEMPERATURE = 0.1
        const val CHAT_TEMPERATURE = 0.3
    }
}
