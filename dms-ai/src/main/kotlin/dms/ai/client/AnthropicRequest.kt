package dms.ai.client

import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.addJsonObject
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonArray
import kotlinx.serialization.json.putJsonObject

/**
 * Turns a chat completions body from [ChatRequestBuilder] into a body for Anthropic's `POST /v1/messages`:
 * system messages move to `system`, image data URLs become base64 image blocks, and the answer schema
 * becomes a structured output. `temperature` is dropped, because current Claude models reject it.
 */
object AnthropicRequest {

    private const val MAX_TOKENS = 16_000
    private val DATA_URL = Regex(pattern = "^data:([^;]+);base64,(.*)$")

    fun of(openAi: JsonObject): JsonObject {
        val (system, chat) = openAi.getValue(key = "messages").jsonArray.map { it.jsonObject }
            .partition { it.getValue(key = "role").jsonPrimitive.content == "system" }
        val schema = openAi["response_format"]?.jsonObject?.get(key = "json_schema")?.jsonObject?.get(key = "schema")
        return buildJsonObject {
            put(key = "model", element = openAi.getValue(key = "model"))
            put(key = "max_tokens", value = MAX_TOKENS)
            if (system.isNotEmpty()) {
                put(key = "system", value = system.joinToString(separator = "\n") { textOf(message = it) })
            }
            putJsonArray(key = "messages") {
                chat.forEach { message ->
                    addJsonObject {
                        put(key = "role", element = message.getValue(key = "role"))
                        put(key = "content", element = contentOf(content = message.getValue(key = "content")))
                    }
                }
            }
            if (schema != null) {
                putJsonObject(key = "output_config") {
                    putJsonObject(key = "format") {
                        put(key = "type", value = "json_schema")
                        put(key = "schema", element = schema)
                    }
                }
            }
        }
    }

    private fun textOf(message: JsonObject): String {
        return message.getValue(key = "content").jsonPrimitive.content
    }

    /** A string stays a string; text parts pass through and `image_url` parts become image blocks. */
    private fun contentOf(content: JsonElement): JsonElement {
        if (content !is JsonArray) {
            return content
        }
        return buildJsonArray {
            content.map { it.jsonObject }.forEach { part ->
                val url = part["image_url"]?.jsonObject?.get(key = "url")?.jsonPrimitive?.content
                val data = url?.let { DATA_URL.find(input = it) }
                if (data == null) {
                    add(element = part)
                } else {
                    addJsonObject {
                        put(key = "type", value = "image")
                        putJsonObject(key = "source") {
                            put(key = "type", value = "base64")
                            put(key = "media_type", value = data.groupValues[1])
                            put(key = "data", value = data.groupValues[2])
                        }
                    }
                }
            }
        }
    }
}
