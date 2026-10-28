package dms.ai.client

import dms.ai.AiAdapters
import dms.ai.support.Requests
import dms.application.port.ai.model.ClassificationFailure
import dms.application.port.ai.model.ClassificationResult
import dms.application.port.ai.model.DocumentContent
import dms.application.port.ai.model.ModelList
import dms.application.port.ai.model.PageImageModel
import dms.application.port.state.model.AiProvider
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
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertTrue

class OpenAiCompatibleClientTest {

    private val settings = AiSettingsModel(baseUrl = "http://localhost:1234/v1/", model = "sample-model")
    private val sentBodies = mutableListOf<JsonObject>()

    @Test
    fun `the request carries the model, the prompt and the answer schema`() {
        runTest {
            val content = """{"title":"Sample bill","tags":["Tax"]}"""
            val client = clientAnswering { json(text = answer(content = content)) }

            val result = client.suggest(request = Requests.request())

            val suggestion = assertIs<ClassificationResult.Suggested>(value = result).suggestion
            assertEquals(expected = "Sample bill", actual = suggestion.title)
            assertEquals(expected = "sample-model", actual = suggestion.model)
            val body = sentBodies.single()
            assertEquals(expected = "sample-model", actual = body.getValue(key = "model").jsonPrimitive.content)
            val format = body.getValue(key = "response_format").jsonObject
            assertEquals(expected = "json_schema", actual = format.getValue(key = "type").jsonPrimitive.content)
            val system = body.getValue(key = "messages").jsonArray.first().jsonObject.getValue(key = "content")
            assertTrue(actual = "Invoices / Energy" in system.jsonPrimitive.content)
        }
    }

    @Test
    fun `page images are sent as data URLs after the text`() {
        runTest {
            val client = clientAnswering { json(text = answer(content = "{}")) }
            val image = PageImageModel(bytes = byteArrayOf(1, 2, 3), mimeType = "image/jpeg")

            client.suggest(request = Requests.request(content = DocumentContent.PageImages(pages = listOf(image))))

            val user = sentBodies.single().getValue(key = "messages").jsonArray[1].jsonObject
            val parts = user.getValue(key = "content").jsonArray.map { it.jsonObject }
            val types = parts.map { it.getValue(key = "type").jsonPrimitive.content }
            assertEquals(expected = listOf("text", "image_url"), actual = types)
            val url = parts[1].getValue(key = "image_url").jsonObject.getValue(key = "url").jsonPrimitive.content
            assertEquals(expected = "data:image/jpeg;base64,AQID", actual = url)
        }
    }

    @Test
    fun `an answer wrapped in a code fence is still read, and prose alone is invalid`() {
        runTest {
            val fenced = clientAnswering { json(text = answer(content = "```json\n{\"title\":\"Sample\"}\n```")) }
            val prose = clientAnswering { json(text = answer(content = "I cannot tell.")) }

            assertIs<ClassificationResult.Suggested>(value = fenced.suggest(request = Requests.request()))
            val failed = assertIs<ClassificationResult.Failed>(value = prose.suggest(request = Requests.request()))
            assertEquals(expected = ClassificationFailure.INVALID_RESPONSE, actual = failed.reason)
        }
    }

    @Test
    fun `a server that cannot be reached is reported as unavailable`() {
        runTest {
            val client = clientAnswering { throw ConnectException("Connection refused") }

            val failed = assertIs<ClassificationResult.Failed>(value = client.suggest(request = Requests.request()))
            val models = assertIs<ModelList.Unavailable>(value = client.models(settings = settings))

            assertEquals(expected = ClassificationFailure.PROVIDER_UNAVAILABLE, actual = failed.reason)
            assertEquals(expected = ClassificationFailure.PROVIDER_UNAVAILABLE, actual = models.reason)
        }
    }

    @Test
    fun `without a chosen model the first loaded one is used`() {
        runTest {
            val client = clientAnswering(settings = settings.copy(model = null)) { request ->
                if (request.url.encodedPath.endsWith(suffix = "/models")) {
                    json(text = """{"data":[{"id":"first-model"},{"id":"second-model"}]}""")
                } else {
                    json(text = answer(content = "{}"))
                }
            }

            val result = client.suggest(request = Requests.request())

            val suggested = assertIs<ClassificationResult.Suggested>(value = result)
            assertEquals(expected = "first-model", actual = suggested.suggestion.model)
        }
    }

    @Test
    fun `a reasoning model's JSON in the reasoning content is used when the answer is empty`() {
        runTest {
            val reasoning = Json.encodeToString(value = """{"title":"Sample bill"}""")
            val body = """{"choices":[{"message":{"role":"assistant","content":"","reasoning_content":$reasoning}}]}"""
            val client = clientAnswering { json(text = body) }

            val result = client.suggest(request = Requests.request())

            assertEquals(
                expected = "Sample bill",
                actual = assertIs<ClassificationResult.Suggested>(value = result).suggestion.title,
            )
        }
    }

    @Test
    fun `Anthropic gets a Messages API request with the key, and its text block is the answer`() {
        runTest {
            val anthropic = AiSettingsModel(provider = AiProvider.ANTHROPIC, anthropicModel = "claude-opus-5-5")
            val requests = mutableListOf<HttpRequestData>()
            val text = Json.encodeToString(value = """{"title":"Sample bill"}""")
            val client = clientAnswering(settings = anthropic, anthropicKey = "sample-key") { request ->
                requests += request
                json(text = """{"content":[{"type":"thinking","thinking":""},{"type":"text","text":$text}]}""")
            }

            val result = client.suggest(request = Requests.request())

            assertEquals(
                expected = "Sample bill",
                actual = assertIs<ClassificationResult.Suggested>(value = result).suggestion.title,
            )
            val request = requests.single()
            assertEquals(expected = "/v1/messages", actual = request.url.encodedPath)
            assertEquals(expected = "sample-key", actual = request.headers["x-api-key"])
            assertEquals(expected = "2023-06-01", actual = request.headers["anthropic-version"])
            assertTrue(actual = "output_config" in sentBodies.single())
        }
    }

    @Test
    fun `Anthropic without an API key fails before any request`() {
        runTest {
            val anthropic = AiSettingsModel(provider = AiProvider.ANTHROPIC, anthropicModel = "claude-opus-5-5")
            val client = clientAnswering(settings = anthropic, anthropicKey = null) { error("No request expected") }

            val failed = assertIs<ClassificationResult.Failed>(value = client.suggest(request = Requests.request()))

            assertEquals(expected = ClassificationFailure.PROVIDER_UNAVAILABLE, actual = failed.reason)
            assertEquals(expected = "ANTHROPIC_API_KEY is not set", actual = failed.detail)
        }
    }

    @Test
    fun `an Anthropic refusal is a failure`() {
        runTest {
            val anthropic = AiSettingsModel(provider = AiProvider.ANTHROPIC, anthropicModel = "claude-opus-5-5")
            val client = clientAnswering(settings = anthropic, anthropicKey = "sample-key") {
                json(text = """{"content":[],"stop_reason":"refusal"}""")
            }

            val failed = assertIs<ClassificationResult.Failed>(value = client.suggest(request = Requests.request()))

            assertEquals(expected = ClassificationFailure.INVALID_RESPONSE, actual = failed.reason)
        }
    }

    @Test
    fun `the Anthropic key is never sent to the local server`() {
        runTest {
            val requests = mutableListOf<HttpRequestData>()
            val local = settings.copy(model = null)
            val client = clientAnswering(settings = local, anthropicKey = "sample-key") { request ->
                requests += request
                if (request.url.encodedPath.endsWith(suffix = "/models")) {
                    json(text = """{"data":[{"id":"first-model"}]}""")
                } else {
                    json(text = answer(content = "{}"))
                }
            }

            client.suggest(request = Requests.request())

            val paths = requests.map { it.url.encodedPath }
            assertEquals(expected = listOf("/v1/models", "/v1/chat/completions"), actual = paths)
            assertTrue(actual = requests.all { it.headers["x-api-key"] == null })
        }
    }

    @Test
    fun `Anthropic's model list is read with the key, and without a key it is unavailable`() {
        runTest {
            val anthropic = AiSettingsModel(provider = AiProvider.ANTHROPIC)
            val requests = mutableListOf<HttpRequestData>()
            val keyed = clientAnswering(settings = anthropic, anthropicKey = "sample-key") { request ->
                requests += request
                json(text = """{"data":[{"id":"claude-opus-5-5"}]}""")
            }
            val keyless = clientAnswering(settings = anthropic, anthropicKey = null) { error("No request expected") }

            val models = assertIs<ModelList.Available>(value = keyed.models(settings = anthropic))
            val missing = assertIs<ModelList.Unavailable>(value = keyless.models(settings = anthropic))

            assertEquals(expected = listOf("claude-opus-5-5"), actual = models.models)
            val request = requests.single()
            assertEquals(expected = "api.anthropic.com", actual = request.url.host)
            assertEquals(expected = "/v1/models", actual = request.url.encodedPath)
            assertEquals(expected = "sample-key", actual = request.headers["x-api-key"])
            assertEquals(expected = "ANTHROPIC_API_KEY is not set", actual = missing.detail)
        }
    }

    private fun clientAnswering(
        settings: AiSettingsModel = this.settings,
        anthropicKey: String? = null,
        handler: suspend MockRequestHandleScope.(HttpRequestData) -> HttpResponseData,
    ): OpenAiCompatibleClient {
        val engine = MockEngine { request ->
            val path = request.url.encodedPath
            if (path.endsWith(suffix = "/chat/completions") || path.endsWith(suffix = "/messages")) {
                sentBodies += Json.parseToJsonElement(string = request.body.toByteArray().decodeToString()).jsonObject
            }
            handler(request)
        }
        return AiAdapters(
            settings = { settings },
            locate = { _, _ -> null },
            engine = engine,
            anthropicKey = { anthropicKey },
        ).client
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
}
