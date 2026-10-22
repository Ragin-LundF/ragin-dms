package dms.ai.assistant

import dms.ai.AiAdapters
import dms.application.port.ai.model.ClassificationFailure
import dms.application.port.ai.model.DocumentContent
import dms.application.port.ai.model.PageImageModel
import dms.application.port.assistant.model.AssistantAnswer
import dms.application.port.assistant.model.AssistantRequestModel
import dms.application.port.assistant.model.ChatRole
import dms.application.port.assistant.model.ChatTurnModel
import dms.application.port.state.model.AiSettingsModel
import dms.domain.taxonomy.model.TagNameModel
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.client.engine.mock.toByteArray
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertTrue

class OpenAiAssistantTest {

    private val sentBodies = mutableListOf<JsonObject>()
    private val text = AssistantRequestModel(
        fileName = "scan-001.pdf",
        content = DocumentContent.Text(text = "Sample Energy, bill 4711"),
    )

    @Test
    fun `a summary sends the summary prompt and the document text`() {
        runTest {
            val assistant = assistantAnswering(content = "<think>hmm</think>A sample bill.")

            val answer = assistant.summarize(request = text)

            assertEquals(expected = "A sample bill.", actual = assertIs<AssistantAnswer.Text>(value = answer).text)
            val messages = messagesOf(body = sentBodies.single())
            assertTrue(actual = "summarize" in contentOf(message = messages[0]))
            val user = contentOf(message = messages[1])
            assertTrue(actual = "Read this document and summarize it." in user)
            assertTrue(actual = "Sample Energy, bill 4711" in user)
        }
    }

    @Test
    fun `tags are read from the answer, and the existing tax tag is named in the prompt`() {
        runTest {
            val assistant = assistantAnswering(content = "Tags: Invoice, tax, Electricity")

            val answer = assistant.suggestTags(request = text, existingTags = setOf(TagNameModel.of(raw = "Tax")))

            val tags = assertIs<AssistantAnswer.Tags>(value = answer).tags.map { it.value }
            assertEquals(expected = listOf("Invoice", "Tax", "Electricity"), actual = tags)
            assertTrue(actual = "include the tag \"Tax\"" in contentOf(message = messagesOf(sentBodies.single())[0]))
        }
    }

    @Test
    fun `an answer without tags is invalid`() {
        runTest {
            val assistant = assistantAnswering(content = "I cannot read this document, it seems to be empty.")

            val answer = assistant.suggestTags(request = text, existingTags = emptySet())

            assertEquals(
                expected = ClassificationFailure.INVALID_RESPONSE,
                actual = assertIs<AssistantAnswer.Failed>(value = answer).reason,
            )
        }
    }

    @Test
    fun `in a conversation the page images go with the first question only`() {
        runTest {
            val assistant = assistantAnswering(content = "On 15 March 2024.")
            val image = PageImageModel(bytes = byteArrayOf(1, 2, 3), mimeType = "image/jpeg")
            val request = AssistantRequestModel(
                fileName = "scan-001.png",
                content = DocumentContent.PageImages(pages = listOf(image)),
            )
            val conversation = listOf(
                ChatTurnModel(role = ChatRole.USER, text = "Who sent it?"),
                ChatTurnModel(role = ChatRole.ASSISTANT, text = "Sample Energy."),
                ChatTurnModel(role = ChatRole.USER, text = "When?"),
            )

            assistant.answer(request = request, conversation = conversation)

            val messages = messagesOf(body = sentBodies.single())
            val roles = messages.map { it.getValue(key = "role").jsonPrimitive.content }
            assertEquals(expected = listOf("system", "user", "assistant", "user"), actual = roles)
            assertIs<JsonArray>(value = messages[1].getValue(key = "content"))
            assertIs<JsonPrimitive>(value = messages[3].getValue(key = "content"))
            assertTrue(actual = "page images" in contentOf(message = messages[0]))
        }
    }

    private fun assistantAnswering(content: String): OpenAiAssistant {
        val engine = MockEngine { request ->
            sentBodies += Json.parseToJsonElement(string = request.body.toByteArray().decodeToString()).jsonObject
            val escaped = Json.encodeToString(value = content)
            respond(
                content = """{"choices":[{"message":{"role":"assistant","content":$escaped}}]}""",
                status = HttpStatusCode.OK,
                headers = headersOf(HttpHeaders.ContentType, "application/json"),
            )
        }
        val settings = AiSettingsModel(model = "sample-model")
        return AiAdapters(settings = { settings }, locate = { _, _ -> null }, engine = engine).assistant
    }

    private fun messagesOf(body: JsonObject): List<JsonObject> {
        return body.getValue(key = "messages").jsonArray.map { it.jsonObject }
    }

    private fun contentOf(message: JsonObject): String {
        return message.getValue(key = "content").jsonPrimitive.content
    }
}
