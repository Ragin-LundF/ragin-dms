package dms.ai.assistant

import dms.ai.AiAdapters
import dms.application.port.ai.model.ClassificationFailure
import dms.application.port.assistant.model.AssistantAnswer
import dms.application.port.assistant.model.SourceChunkModel
import dms.application.port.state.model.AiSettingsModel
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.MockRequestHandleScope
import io.ktor.client.engine.mock.respond
import io.ktor.client.engine.mock.toByteArray
import io.ktor.client.request.HttpRequestData
import io.ktor.client.request.HttpResponseData
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import java.net.ConnectException
import java.net.SocketTimeoutException
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertTrue

class OpenAiLibraryAssistantTest {

    private val sentBodies = mutableListOf<JsonObject>()
    private val settings = AiSettingsModel(model = "sample-chat")
    private val sources = listOf(SourceChunkModel(number = 1, title = "Sample bill", text = "Meter: 120 kWh"))

    @Test
    fun `the answer comes without thoughts and emphasis, with the model that gave it`() {
        runTest {
            val assistant = assistantAnswering {
                json(text = answer(content = "<think>look</think>It was **120 kWh** [1]."))
            }

            val answer = assistant.answer(question = "How much power?", sources = sources)

            assertEquals(
                expected = AssistantAnswer.Text(text = "It was 120 kWh [1].", model = "sample-chat"),
                actual = answer,
            )
        }
    }

    @Test
    fun `every passage is sent with its number, cut to its maximum length`() {
        runTest {
            val assistant = assistantAnswering { json(text = answer(content = "Done [2].")) }
            val long = SourceChunkModel(number = 2, title = "Sample contract", text = "y".repeat(n = 5_000))

            assistant.answer(question = "What does it say?", sources = sources + long)

            val system = contentOf(message = messagesOf(body = sentBodies.single()).first())
            assertTrue(actual = "[1] Meter: 120 kWh\n\n[2] ${"y".repeat(n = 4_000)}" in system)
            assertFalse(actual = "y".repeat(n = 4_001) in system)
        }
    }

    @Test
    fun `the library assistant gets the numbered passages and the question`() {
        runTest {
            val assistant = assistantAnswering { json(text = answer(content = "It was 120 kWh [1].")) }

            val answer = assistant.answer(question = "How much power?", sources = sources)

            assertEquals(
                expected = AssistantAnswer.Text(text = "It was 120 kWh [1].", model = "sample-chat"),
                actual = answer,
            )
            val messages = messagesOf(body = sentBodies.single())
            assertTrue(actual = "[1] Meter: 120 kWh" in contentOf(message = messages.first()))
            assertEquals(expected = "How much power?", actual = contentOf(message = messages.last()))
        }
    }

    @Test
    fun `an empty answer, one with thoughts only, or one without choices is invalid`() {
        runTest {
            val answers = listOf(
                answer(content = ""),
                answer(content = "<think>The passages say nothing.</think>"),
                """{"choices":[]}""",
                """{"choices":[{"message":{"role":"assistant"}}]}""",
            )

            answers.forEach { text ->
                val assistant = assistantAnswering { json(text = text) }

                val answer = assistant.answer(question = "How much power?", sources = sources)

                assertEquals(
                    expected = ClassificationFailure.INVALID_RESPONSE,
                    actual = assertIs<AssistantAnswer.Failed>(value = answer, message = text).reason,
                    message = text,
                )
            }
        }
    }

    @Test
    fun `an answer that is no JSON or not the chat schema is invalid`() {
        runTest {
            val answers = listOf(
                "<html>Internal error</html>",
                "",
                """{"choices":{"message":"It was 120 kWh."}}""",
                """{"choices":[{"message":{"role":"assistant","content":["It was 120 kWh."]}}]}""",
            )

            answers.forEach { text ->
                val assistant = assistantAnswering { json(text = text) }

                val answer = assistant.answer(question = "How much power?", sources = sources)

                assertEquals(
                    expected = ClassificationFailure.INVALID_RESPONSE,
                    actual = assertIs<AssistantAnswer.Failed>(value = answer, message = text).reason,
                    message = text,
                )
            }
        }
    }

    @Test
    fun `a refused request is an unavailable provider, never an image problem`() {
        runTest {
            val assistant = assistantAnswering {
                respond(content = "context too long", status = HttpStatusCode.BadRequest)
            }

            val answer = assistant.answer(question = "How much power?", sources = sources)

            assertEquals(
                expected = AssistantAnswer.Failed(
                    reason = ClassificationFailure.PROVIDER_UNAVAILABLE,
                    detail = "HTTP 400: context too long",
                ),
                actual = answer,
            )
        }
    }

    @Test
    fun `an unreachable server and a server that takes too long are told apart`() {
        runTest {
            val unreachable = assistantAnswering { throw ConnectException("Connection refused") }
            val slow = assistantAnswering { throw SocketTimeoutException("Read timed out") }

            val refused = unreachable.answer(question = "How much power?", sources = sources)
            val timedOut = slow.answer(question = "How much power?", sources = sources)

            assertEquals(
                expected = ClassificationFailure.PROVIDER_UNAVAILABLE,
                actual = assertIs<AssistantAnswer.Failed>(value = refused).reason,
            )
            assertEquals(
                expected = ClassificationFailure.TIMEOUT,
                actual = assertIs<AssistantAnswer.Failed>(value = timedOut).reason,
            )
        }
    }

    @Test
    fun `without a chat model on the server the question is not sent`() {
        runTest {
            val assistant = assistantAnswering(settings = AiSettingsModel()) {
                json(text = """{"data":[{"id":"text-embedding-sample"}]}""")
            }

            val answer = assistant.answer(question = "How much power?", sources = sources)

            assertEquals(
                expected = ClassificationFailure.PROVIDER_UNAVAILABLE,
                actual = assertIs<AssistantAnswer.Failed>(value = answer).reason,
            )
            assertTrue(actual = sentBodies.isEmpty())
        }
    }

    private fun assistantAnswering(
        settings: AiSettingsModel = this.settings,
        handler: suspend MockRequestHandleScope.(HttpRequestData) -> HttpResponseData,
    ): OpenAiLibraryAssistant {
        val engine = MockEngine { request ->
            if (request.url.encodedPath.endsWith(suffix = "/chat/completions")) {
                sentBodies += Json.parseToJsonElement(string = request.body.toByteArray().decodeToString()).jsonObject
            }
            handler(request)
        }
        return AiAdapters(settings = { settings }, locate = { _, _ -> null }, engine = engine).libraryAssistant
    }

    private fun MockRequestHandleScope.json(text: String): HttpResponseData {
        return respond(
            content = text,
            status = HttpStatusCode.OK,
            headers = headersOf(HttpHeaders.ContentType, "application/json"),
        )
    }

    private fun answer(content: String): String {
        val escaped = Json.encodeToString(value = content)
        return """{"choices":[{"message":{"role":"assistant","content":$escaped}}]}"""
    }

    private fun messagesOf(body: JsonObject): List<JsonObject> {
        return body.getValue(key = "messages").jsonArray.map { it.jsonObject }
    }

    private fun contentOf(message: JsonObject): String {
        return message.getValue(key = "content").jsonPrimitive.content
    }
}
