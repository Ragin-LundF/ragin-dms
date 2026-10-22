package dms.ai.client

import dms.ai.client.model.ChatMessageModel
import dms.ai.prompt.PromptTexts
import dms.ai.prompt.SuggestionSchema
import dms.application.port.ai.model.ClassificationRequestModel
import dms.application.port.ai.model.DocumentContent
import dms.application.port.ai.model.PageImageModel
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonObjectBuilder
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.addJsonObject
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonArray
import kotlinx.serialization.json.putJsonObject
import java.util.Base64

/** Builds the body of `POST /chat/completions`: the messages, and for a classification the answer schema. */
object ChatRequestBuilder {

    private const val CLASSIFICATION_TEMPERATURE = 0.1

    /** A classification: the rules and taxonomy, the document, and the JSON schema of the answer. */
    fun build(model: String, request: ClassificationRequestModel): JsonObject {
        val images = (request.content as? DocumentContent.PageImages)?.pages.orEmpty()
        val text = when (val content = request.content) {
            is DocumentContent.Text -> PromptTexts.user(request = request, text = content.text)
            is DocumentContent.NotReadable -> PromptTexts.user(request = request, text = "")
            is DocumentContent.PageImages -> PromptTexts.user(request = request, text = null)
        }
        val messages = listOf(
            ChatMessageModel(role = "system", content = JsonPrimitive(value = PromptTexts.system(request = request))),
            ChatMessageModel(role = "user", content = content(text = text, images = images)),
        )
        return buildJsonObject {
            fill(model = model, temperature = CLASSIFICATION_TEMPERATURE, messages = messages)
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

    /** A plain chat, answered with free text. */
    fun chat(model: String, temperature: Double, messages: List<ChatMessageModel>): JsonObject {
        return buildJsonObject {
            fill(model = model, temperature = temperature, messages = messages)
        }
    }

    /** Text alone as one string; with page images, a text part followed by one image part per page. */
    fun content(text: String, images: List<PageImageModel>): JsonElement {
        if (images.isEmpty()) {
            return JsonPrimitive(value = text)
        }
        return buildJsonArray {
            addJsonObject {
                put(key = "type", value = "text")
                put(key = "text", value = text)
            }
            images.forEach { page ->
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

    private fun JsonObjectBuilder.fill(model: String, temperature: Double, messages: List<ChatMessageModel>) {
        put(key = "model", value = model)
        put(key = "temperature", value = temperature)
        put(key = "stream", value = false)
        putJsonArray(key = "messages") {
            messages.forEach { message ->
                addJsonObject {
                    put(key = "role", value = message.role)
                    put(key = "content", element = message.content)
                }
            }
        }
    }
}
