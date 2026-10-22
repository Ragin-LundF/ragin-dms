package dms.ai.client

import dms.ai.AiAdapters
import dms.application.port.ai.model.ClassificationFailure
import dms.application.port.ai.model.EmbeddingResult
import dms.application.port.assistant.model.AssistantAnswer
import dms.application.port.assistant.model.SourceChunkModel
import dms.application.port.state.model.AiSettingsModel
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.client.engine.mock.toByteArray
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertTrue

class OpenAiEmbeddingsTest {

    private val bodies = mutableListOf<String>()

    private fun adapters(settings: AiSettingsModel, answer: (String) -> Pair<HttpStatusCode, String>): AiAdapters {
        val engine = MockEngine { request ->
            bodies += request.body.toByteArray().decodeToString()
            val (status, body) = answer(request.url.encodedPath)
            respond(content = body, status = status, headers = headersOf(HttpHeaders.ContentType, "application/json"))
        }
        return AiAdapters(settings = { settings }, locate = { _, _ -> null }, engine = engine)
    }

    @Test
    fun `texts are embedded with the chosen model and returned in their order`() {
        runTest {
            val ai = adapters(settings = AiSettingsModel(embeddingModel = "sample-embed")) {
                val data = """[{"index":1,"embedding":[0.5,0.5]},{"index":0,"embedding":[1.0,0.0]}]"""
                HttpStatusCode.OK to """{"data":$data}"""
            }

            val result = assertIs<EmbeddingResult.Vectors>(
                value = ai.embeddings.embed(texts = listOf("first", "second")),
            )

            assertEquals(expected = "sample-embed", actual = result.model)
            assertContentEquals(expected = floatArrayOf(1f, 0f), actual = result.vectors[0])
            val sent = Json.parseToJsonElement(string = bodies.single()).jsonObject
            assertEquals(expected = "sample-embed", actual = sent.getValue(key = "model").jsonPrimitive.content)
            assertEquals(expected = 2, actual = sent.getValue(key = "input").jsonArray.size)
        }
    }

    @Test
    fun `without a chosen model the first embedding model of the server is used`() {
        runTest {
            val ai = adapters(settings = AiSettingsModel()) { path ->
                if (path.endsWith(suffix = "/models")) {
                    HttpStatusCode.OK to """{"data":[{"id":"chat-model"},{"id":"text-embedding-sample"}]}"""
                } else {
                    HttpStatusCode.OK to """{"data":[{"index":0,"embedding":[1.0]}]}"""
                }
            }

            assertEquals(expected = "text-embedding-sample", actual = ai.embeddings.model())
        }
    }

    @Test
    fun `a refusing server or a wrong number of vectors is a failure`() {
        runTest {
            val settings = AiSettingsModel(embeddingModel = "e")
            val refusing = adapters(settings = settings) { HttpStatusCode.NotFound to "{}" }
            val short = adapters(settings = settings) { HttpStatusCode.OK to """{"data":[]}""" }

            assertEquals(
                expected = ClassificationFailure.PROVIDER_UNAVAILABLE,
                actual = (refusing.embeddings.embed(texts = listOf("a")) as EmbeddingResult.Failed).reason,
            )
            assertEquals(
                expected = ClassificationFailure.INVALID_RESPONSE,
                actual = (short.embeddings.embed(texts = listOf("a")) as EmbeddingResult.Failed).reason,
            )
        }
    }

    @Test
    fun `the library assistant gets the numbered passages and the question`() {
        runTest {
            val ai = adapters(settings = AiSettingsModel(model = "chat")) {
                val message = """{"role":"assistant","content":"It was 120 kWh [1]."}"""
                HttpStatusCode.OK to """{"choices":[{"message":$message}]}"""
            }

            val answer = ai.libraryAssistant.answer(
                question = "How much power?",
                sources = listOf(SourceChunkModel(number = 1, title = "Bill", text = "Meter: 120 kWh")),
            )

            assertEquals(expected = AssistantAnswer.Text(text = "It was 120 kWh [1].", model = "chat"), actual = answer)
            val messages = Json.parseToJsonElement(
                string = bodies.single(),
            ).jsonObject.getValue(key = "messages").jsonArray
            assertTrue(
                actual = messages.first().jsonObject.getValue(
                    key = "content",
                ).jsonPrimitive.content.contains(other = "[1] Meter: 120 kWh"),
            )
            assertEquals(
                expected = "How much power?",
                actual = messages.last().jsonObject.getValue(key = "content").jsonPrimitive.content,
            )
        }
    }
}
