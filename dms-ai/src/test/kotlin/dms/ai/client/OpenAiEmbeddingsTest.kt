package dms.ai.client

import dms.ai.AiAdapters
import dms.application.port.ai.model.ClassificationFailure
import dms.application.port.ai.model.EmbeddingResult
import dms.application.port.state.model.AiProvider
import dms.application.port.state.model.AiSettingsModel
import io.ktor.client.request.HttpRequestData
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
import java.net.ConnectException
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNull

class OpenAiEmbeddingsTest {

    private val bodies = mutableListOf<String>()
    private val paths = mutableListOf<String>()
    private val requests = mutableListOf<HttpRequestData>()

    private fun adapters(settings: AiSettingsModel, answer: (String) -> Pair<HttpStatusCode, String>): AiAdapters {
        val engine = MockEngine { request ->
            bodies += request.body.toByteArray().decodeToString()
            paths += request.url.encodedPath
            requests += request
            val (status, body) = answer(request.url.encodedPath)
            respond(content = body, status = status, headers = headersOf(HttpHeaders.ContentType, "application/json"))
        }
        return AiAdapters(
            settings = { settings },
            locate = { _, _ -> null },
            engine = engine,
            anthropicKey = { "sample-key" },
        )
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
    fun `a blank chosen model falls back to the server's embedding model, whatever its case`() {
        runTest {
            val ai = adapters(settings = AiSettingsModel(embeddingModel = "  ")) {
                HttpStatusCode.OK to """{"data":[{"id":"chat-model"},{"id":"Sample-EMBED-Text"}]}"""
            }

            assertEquals(expected = "Sample-EMBED-Text", actual = ai.embeddings.model())
        }
    }

    @Test
    fun `without any embedding model the texts are not sent and the provider is unavailable`() {
        runTest {
            val ai = adapters(settings = AiSettingsModel()) {
                HttpStatusCode.OK to """{"data":[{"id":"chat-model"}]}"""
            }

            val result = assertIs<EmbeddingResult.Failed>(value = ai.embeddings.embed(texts = listOf("a")))

            assertEquals(expected = ClassificationFailure.PROVIDER_UNAVAILABLE, actual = result.reason)
            assertEquals(expected = listOf("/v1/models"), actual = paths)
        }
    }

    @Test
    fun `a server that cannot be reached is unavailable, with or without a chosen model`() {
        runTest {
            val unreachable: (String) -> Pair<HttpStatusCode, String> = { throw ConnectException("Connection refused") }
            val chosen = adapters(settings = AiSettingsModel(embeddingModel = "e"), answer = unreachable)
            val listed = adapters(settings = AiSettingsModel(), answer = unreachable)

            val failed = assertIs<EmbeddingResult.Failed>(value = chosen.embeddings.embed(texts = listOf("a")))

            assertEquals(
                expected = EmbeddingResult.Failed(
                    reason = ClassificationFailure.PROVIDER_UNAVAILABLE,
                    detail = "Connection refused",
                ),
                actual = failed,
            )
            assertEquals(
                expected = ClassificationFailure.PROVIDER_UNAVAILABLE,
                actual = assertIs<EmbeddingResult.Failed>(value = listed.embeddings.embed(texts = listOf("a"))).reason,
            )
        }
    }

    @Test
    fun `a refusing server's status and a shortened answer are the detail`() {
        runTest {
            val ai = adapters(settings = AiSettingsModel(embeddingModel = "e")) {
                HttpStatusCode.ServiceUnavailable to "x".repeat(n = 1_000)
            }

            val failed = assertIs<EmbeddingResult.Failed>(value = ai.embeddings.embed(texts = listOf("a")))

            assertEquals(
                expected = "HTTP 503: " + "x".repeat(n = ChatTransport.DETAIL_LENGTH),
                actual = failed.detail,
            )
        }
    }

    @Test
    fun `a missing or empty vector makes the whole answer invalid, not a partial result`() {
        runTest {
            val answers = listOf(
                """{"data":[{"index":0,"embedding":[1.0]},{"index":1}]}""",
                """{"data":[{"index":0,"embedding":[1.0]},{"index":1,"embedding":[]}]}""",
                """{"data":[{"index":0,"embedding":[1.0]}]}""",
                """{"model":"e"}""",
            )

            answers.forEach { answer ->
                val ai = adapters(settings = AiSettingsModel(embeddingModel = "e")) { HttpStatusCode.OK to answer }

                val result = ai.embeddings.embed(texts = listOf("first", "second"))

                assertEquals(
                    expected = ClassificationFailure.INVALID_RESPONSE,
                    actual = assertIs<EmbeddingResult.Failed>(value = result, message = answer).reason,
                    message = answer,
                )
            }
        }
    }

    @Test
    fun `a base URL ending with a slash still reaches the embeddings endpoint`() {
        runTest {
            val settings = AiSettingsModel(baseUrl = " http://localhost:1234/v1/ ", embeddingModel = "e")
            val ai = adapters(settings = settings) {
                HttpStatusCode.OK to """{"data":[{"index":0,"embedding":[1.0]}]}"""
            }

            assertIs<EmbeddingResult.Vectors>(value = ai.embeddings.embed(texts = listOf("a")))

            assertEquals(expected = listOf("/v1/embeddings"), actual = paths)
        }
    }

    @Test
    fun `an answer that is no JSON or not the embeddings schema is invalid`() {
        runTest {
            val answers = listOf(
                "<html>Internal error</html>",
                "",
                """{"data":{"index":0,"embedding":[1.0]}}""",
                """{"data":[{"index":0,"embedding":["one"]}]}""",
                """{"data":[{"index":"first","embedding":[1.0]}]}""",
            )

            answers.forEach { answer ->
                val ai = adapters(settings = AiSettingsModel(embeddingModel = "e")) { HttpStatusCode.OK to answer }

                val result = ai.embeddings.embed(texts = listOf("a"))

                assertEquals(
                    expected = ClassificationFailure.INVALID_RESPONSE,
                    actual = assertIs<EmbeddingResult.Failed>(value = result, message = answer).reason,
                    message = answer,
                )
            }
        }
    }

    @Test
    fun `vectors that cannot be matched to the texts make the answer invalid`() {
        runTest {
            val answers = listOf(
                """{"data":[{"index":0,"embedding":[1.0]},{"index":0,"embedding":[2.0]}]}""",
                """{"data":[{"embedding":[1.0]},{"embedding":[2.0]}]}""",
                """{"data":[{"index":1,"embedding":[1.0]},{"index":2,"embedding":[2.0]}]}""",
                """{"data":[{"index":0,"embedding":[1.0,2.0]},{"index":1,"embedding":[2.0]}]}""",
            )

            answers.forEach { answer ->
                val ai = adapters(settings = AiSettingsModel(embeddingModel = "e")) { HttpStatusCode.OK to answer }

                val result = ai.embeddings.embed(texts = listOf("first", "second"))

                assertEquals(
                    expected = ClassificationFailure.INVALID_RESPONSE,
                    actual = assertIs<EmbeddingResult.Failed>(value = result, message = answer).reason,
                    message = answer,
                )
            }
        }
    }

    @Test
    fun `while Anthropic answers the chats, embeddings go to the local server without the key`() {
        runTest {
            val settings = AiSettingsModel(provider = AiProvider.ANTHROPIC, embeddingModel = "sample-embed")
            val ai = adapters(settings = settings) {
                HttpStatusCode.OK to """{"data":[{"index":0,"embedding":[1.0]}]}"""
            }

            assertIs<EmbeddingResult.Vectors>(value = ai.embeddings.embed(texts = listOf("first")))

            val request = requests.single()
            assertEquals(expected = "localhost", actual = request.url.host)
            assertEquals(expected = "/v1/embeddings", actual = request.url.encodedPath)
            assertNull(actual = request.headers["x-api-key"])
        }
    }
}
