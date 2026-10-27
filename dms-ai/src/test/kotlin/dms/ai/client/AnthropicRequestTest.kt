package dms.ai.client

import dms.ai.client.model.ChatMessageModel
import dms.application.port.ai.model.PageImageModel
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonObject
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse

class AnthropicRequestTest {

    @Test
    fun `system messages move to system, images become base64 blocks, and temperature is dropped`() {
        val image = PageImageModel(bytes = byteArrayOf(1, 2, 3), mimeType = "image/jpeg")
        val openAi = ChatRequestBuilder.chat(
            model = "claude-opus-5-5",
            temperature = 0.2,
            messages = listOf(
                ChatMessageModel(role = "system", content = JsonPrimitive(value = "Be brief.")),
                ChatMessageModel(
                    role = "user",
                    content = ChatRequestBuilder.content(text = "Read", images = listOf(image)),
                ),
            ),
        )

        val body = AnthropicRequest.of(openAi = openAi)

        assertEquals(expected = "Be brief.", actual = body.getValue(key = "system").jsonPrimitive.content)
        assertFalse(actual = "temperature" in body)
        assertEquals(expected = 16_000, actual = body.getValue(key = "max_tokens").jsonPrimitive.content.toInt())
        val message = body.getValue(key = "messages").jsonArray.single().jsonObject
        assertEquals(expected = "user", actual = message.getValue(key = "role").jsonPrimitive.content)
        val source = message.getValue(key = "content").jsonArray[1].jsonObject.getValue(key = "source").jsonObject
        assertEquals(expected = "image/jpeg", actual = source.getValue(key = "media_type").jsonPrimitive.content)
        assertEquals(expected = "AQID", actual = source.getValue(key = "data").jsonPrimitive.content)
    }

    @Test
    fun `the answer schema becomes a structured output`() {
        val openAi = buildJsonObject {
            put(key = "model", value = "claude-opus-5-5")
            put(key = "messages", element = JsonArray(content = emptyList()))
            putJsonObject(key = "response_format") {
                putJsonObject(key = "json_schema") {
                    putJsonObject(key = "schema") { put(key = "type", value = "object") }
                }
            }
        }

        val format = AnthropicRequest.of(openAi = openAi).getValue(key = "output_config").jsonObject
            .getValue(key = "format").jsonObject

        assertEquals(expected = "json_schema", actual = format.getValue(key = "type").jsonPrimitive.content)
        val schema = format.getValue(key = "schema").jsonObject
        assertEquals(expected = "object", actual = schema.getValue(key = "type").jsonPrimitive.content)
    }
}
