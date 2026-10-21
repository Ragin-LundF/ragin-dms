package dms.ai.client

import dms.ai.AiAdapters
import dms.ai.support.Requests
import dms.application.port.ai.model.ClassificationFailure
import dms.application.port.ai.model.ClassificationResult
import dms.application.port.ai.model.DocumentContent
import dms.application.port.ai.model.ModelList
import dms.application.port.ai.model.PageImage
import dms.application.port.state.model.AiSettings
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

    private val settings = AiSettings(baseUrl = "http://localhost:1234/v1/", model = "sample-model")
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
            val image = PageImage(bytes = byteArrayOf(1, 2, 3), mimeType = "image/jpeg")

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

    private fun clientAnswering(
        settings: AiSettings = this.settings,
        handler: suspend MockRequestHandleScope.(HttpRequestData) -> HttpResponseData,
    ): OpenAiCompatibleClient {
        val engine = MockEngine { request ->
            if (request.url.encodedPath.endsWith(suffix = "/chat/completions")) {
                sentBodies += Json.parseToJsonElement(string = request.body.toByteArray().decodeToString()).jsonObject
            }
            handler(request)
        }
        return AiAdapters(settings = { settings }, locate = { _, _ -> null }, engine = engine).client
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
