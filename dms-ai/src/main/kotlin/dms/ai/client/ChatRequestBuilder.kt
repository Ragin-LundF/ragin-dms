package dms.ai.client

import dms.ai.prompt.PromptTexts
import dms.ai.prompt.SuggestionSchema
import dms.application.port.ai.model.ClassificationRequest
import dms.application.port.ai.model.DocumentContent
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.addJsonObject
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonArray
import kotlinx.serialization.json.putJsonObject
import java.util.Base64

/** Builds the body of `POST /chat/completions`: the prompt, the document, and the answer schema. */
object ChatRequestBuilder {

    private const val TEMPERATURE = 0.1

    fun build(model: String, request: ClassificationRequest): JsonObject {
        return buildJsonObject {
            put(key = "model", value = model)
            put(key = "temperature", value = TEMPERATURE)
            put(key = "stream", value = false)
            putJsonArray(key = "messages") {
                addJsonObject {
                    put(key = "role", value = "system")
                    put(key = "content", value = PromptTexts.system(request = request))
                }
                addJsonObject {
                    put(key = "role", value = "user")
                    put(key = "content", element = userContent(request = request))
                }
            }
            putJsonObject(key = "response_format") {
                put(key = "type", value = "json_schema")
                putJsonObject(key = "json_schema") {
                    put(key = "name", value = "document_classification")
                    put(key = "strict", value = true)
                    put(key = "schema", element = SuggestionSchema.schema)
                }
            }
        }
    }

    /** Text as one string; page images as a text part followed by one image part per page. */
    private fun userContent(request: ClassificationRequest): JsonElement {
        return when (val content = request.content) {
            is DocumentContent.Text -> JsonPrimitive(value = PromptTexts.user(request = request, text = content.text))
            is DocumentContent.NotReadable -> JsonPrimitive(value = PromptTexts.user(request = request, text = ""))
            is DocumentContent.PageImages -> buildJsonArray {
                addJsonObject {
                    put(key = "type", value = "text")
                    put(key = "text", value = PromptTexts.user(request = request, text = null))
                }
                content.pages.forEach { page ->
                    addJsonObject {
                        put(key = "type", value = "image_url")
                        putJsonObject(key = "image_url") {
                            val data = Base64.getEncoder().encodeToString(page.bytes)
                            put(key = "url", value = "data:${page.mimeType};base64,$data")
                        }
                    }
                }
            }
        }
    }
}
