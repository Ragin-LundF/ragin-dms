package dms.ai.assistant

import dms.ai.client.AnswerText
import dms.ai.client.ChatRequestBuilder
import dms.ai.client.ChatTransport
import dms.ai.client.model.ChatMessageModel
import dms.ai.client.model.ChatReply
import dms.ai.prompt.PromptTemplates
import dms.application.port.assistant.LibraryAssistant
import dms.application.port.assistant.model.AssistantAnswer
import dms.application.port.assistant.model.SourceChunkModel
import kotlinx.serialization.json.JsonPrimitive

/** Answers questions about the whole library from the passages found, on the chat model. */
class OpenAiLibraryAssistant(private val transport: ChatTransport) : LibraryAssistant {

    override suspend fun answer(question: String, sources: List<SourceChunkModel>): AssistantAnswer {
        val passages = sources.joinToString(separator = "\n\n") { "[${it.number}] ${it.text.take(n = MAX_PASSAGE)}" }
        val messages = listOf(
            ChatMessageModel(
                role = "system",
                content = JsonPrimitive(
                    value = PromptTemplates.fill(name = "ask-library.md", values = mapOf("sources" to passages)),
                ),
            ),
            ChatMessageModel(role = "user", content = JsonPrimitive(value = question)),
        )
        val reply = transport.complete(hasImages = false) { model ->
            ChatRequestBuilder.chat(model = model, temperature = TEMPERATURE, messages = messages)
        }
        return when (reply) {
            is ChatReply.Answered -> AssistantAnswer.Text(
                text = AnswerText.plain(content = reply.content),
                model = reply.model,
            )
            is ChatReply.Failed -> AssistantAnswer.Failed(reason = reply.reason, detail = reply.detail)
        }
    }

    private companion object {
        const val TEMPERATURE = 0.2
        const val MAX_PASSAGE = 4_000
    }
}
