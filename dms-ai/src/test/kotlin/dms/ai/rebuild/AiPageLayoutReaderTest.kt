package dms.ai.rebuild

import dms.ai.AiAdapters
import dms.application.port.ai.model.ClassificationFailure
import dms.application.port.ai.model.PageImageModel
import dms.application.port.ai.model.layout.LayoutOutcome
import dms.application.port.ai.model.layout.LineOutcome
import dms.application.port.state.model.AiSettingsModel
import dms.domain.rebuild.FontFamily
import dms.domain.rebuild.model.LayoutBoxModel
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.MockRequestHandleScope
import io.ktor.client.engine.mock.respond
import io.ktor.client.engine.mock.toByteArray
import io.ktor.client.plugins.HttpRequestTimeoutException
import io.ktor.client.plugins.HttpTimeoutCapability
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
import java.awt.image.BufferedImage
import java.io.ByteArrayOutputStream
import javax.imageio.ImageIO
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertTrue

class AiPageLayoutReaderTest {

    private val settings = AiSettingsModel(model = "sample-model", timeoutSeconds = 60)
    private val sentBodies = mutableListOf<JsonObject>()
    private val sentTimeouts = mutableListOf<Long?>()

    /** A synthetic page of 2000 x 1000 pixels; it goes out as 1568 x 784. */
    private val page = PageImageModel(bytes = png(width = 2000, height = 1000), mimeType = "image/png")

    @Test
    fun `an answer in 0 to 1000 keeps its boxes and its umlauts`() {
        runTest {
            val content = """{"lines":[{"bbox":[100,200,500,240],"text":"Prüfung für Müller","bold":true,""" +
                """"font":"serif","legible":true}],"graphics":[{"kind":"logo","bbox":[800,50,950,150]}]}"""
            val reader = readerAnswering { json(text = answer(content = content)) }

            val layout = assertIs<LayoutOutcome.Read>(value = reader.readLayout(page = page)).layout

            val line = layout.lines.single()
            assertEquals(
                expected = LayoutBoxModel(left = 100.0, top = 200.0, right = 500.0, bottom = 240.0),
                actual = line.box,
            )
            assertEquals(expected = "Prüfung für Müller", actual = line.text)
            assertEquals(expected = FontFamily.SERIF, actual = line.family)
            assertTrue(actual = line.bold && line.legible)
            assertEquals(expected = "logo", actual = layout.graphics.single().kind)
        }
    }

    @Test
    fun `an answer in pixels of the sent image is converted to 0 to 1000`() {
        runTest {
            val content = """{"lines":[{"bbox":[784,392,1568,784],"text":"Beispiel","bold":false,"font":"sans",""" +
                """"legible":true}],"graphics":[]}"""
            val reader = readerAnswering { json(text = answer(content = content)) }

            val layout = assertIs<LayoutOutcome.Read>(value = reader.readLayout(page = page)).layout

            assertEquals(
                expected = LayoutBoxModel(left = 500.0, top = 500.0, right = 1000.0, bottom = 1000.0),
                actual = layout.lines.single().box,
            )
        }
    }

    @Test
    fun `the page goes out as PNG on the patch grid with the schema and a long timeout`() {
        runTest {
            val reader = readerAnswering { json(text = answer(content = EMPTY_LAYOUT)) }

            reader.readLayout(page = page)

            val body = sentBodies.single()
            val format = body.getValue(key = "response_format").jsonObject
            val schema = format.getValue(key = "json_schema").jsonObject
            val schemaName = schema.getValue(key = "name").jsonPrimitive.content
            assertEquals(expected = "page_layout", actual = schemaName)
            val messages = body.getValue(key = "messages").jsonArray
            val user = messages[1].jsonObject.getValue(key = "content").jsonArray
            assertTrue(actual = "1568 x 784" in user[0].jsonObject.getValue(key = "text").jsonPrimitive.content)
            val image = user[1].jsonObject.getValue(key = "image_url").jsonObject
            val url = image.getValue(key = "url").jsonPrimitive.content
            assertTrue(actual = url.startsWith(prefix = "data:image/png;base64,"))
            assertEquals(expected = 600_000L, actual = sentTimeouts.single())
        }
    }

    @Test
    fun `an answer only in the reasoning content is read`() {
        runTest {
            val escaped = Json.encodeToString(value = EMPTY_LAYOUT)
            val reader = readerAnswering {
                json(text = """{"choices":[{"message":{"content":"","reasoning_content":$escaped}}]}""")
            }

            assertIs<LayoutOutcome.Read>(value = reader.readLayout(page = page))
        }
    }

    @Test
    fun `a thinking block and a code fence around the answer are dropped`() {
        runTest {
            val reader = readerAnswering {
                json(text = answer(content = "<think>looking at the page</think>\n```json\n$EMPTY_LAYOUT\n```"))
            }

            assertIs<LayoutOutcome.Read>(value = reader.readLayout(page = page))
        }
    }

    @Test
    fun `HTTP 400 on an image means the model cannot read images`() {
        runTest {
            val reader = readerAnswering { respond(content = "no vision", status = HttpStatusCode.BadRequest) }

            val failure = assertIs<LayoutOutcome.Failure>(value = reader.readLayout(page = page))

            assertEquals(expected = ClassificationFailure.MODEL_CANNOT_READ_IMAGES, actual = failure.reason)
        }
    }

    @Test
    fun `a request that runs out of time is a timeout`() {
        runTest {
            val reader = readerAnswering { request -> throw HttpRequestTimeoutException(request = request) }

            val failure = assertIs<LayoutOutcome.Failure>(value = reader.readLayout(page = page))

            assertEquals(expected = ClassificationFailure.TIMEOUT, actual = failure.reason)
        }
    }

    @Test
    fun `broken JSON is an invalid answer`() {
        runTest {
            val reader = readerAnswering { json(text = answer(content = """{"lines":[{"bbox":[1,2""")) }

            val failure = assertIs<LayoutOutcome.Failure>(value = reader.readLayout(page = page))

            assertEquals(expected = ClassificationFailure.INVALID_RESPONSE, actual = failure.reason)
        }
    }

    @Test
    fun `a line is read again as plain text without a schema`() {
        runTest {
            val reader = readerAnswering { json(text = answer(content = "Here it is:\n```\nGrüße aus Köln\n```")) }

            val read = assertIs<LineOutcome.Read>(value = reader.readLine(line = page))

            assertEquals(expected = "Grüße aus Köln", actual = read.text)
            assertFalse(actual = "response_format" in sentBodies.single())
        }
    }

    private fun readerAnswering(
        handler: suspend MockRequestHandleScope.(HttpRequestData) -> HttpResponseData,
    ): AiPageLayoutReader {
        val engine = MockEngine { request ->
            sentBodies += Json.parseToJsonElement(string = request.body.toByteArray().decodeToString()).jsonObject
            sentTimeouts += request.getCapabilityOrNull(key = HttpTimeoutCapability)?.requestTimeoutMillis
            handler(request)
        }
        return AiAdapters(settings = { settings }, locate = { _, _ -> null }, engine = engine).layoutReader
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

    private fun png(width: Int, height: Int): ByteArray {
        val output = ByteArrayOutputStream()
        ImageIO.write(BufferedImage(width, height, BufferedImage.TYPE_INT_RGB), "png", output)
        return output.toByteArray()
    }

    private companion object {
        const val EMPTY_LAYOUT = """{"lines":[],"graphics":[]}"""
    }
}
