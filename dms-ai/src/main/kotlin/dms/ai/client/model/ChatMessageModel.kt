package dms.ai.client.model

import kotlinx.serialization.json.JsonElement

/** One message of a chat request; [content] is a string or a list of text and image parts. */
data class ChatMessageModel(
    val role: String,
    val content: JsonElement,
)
