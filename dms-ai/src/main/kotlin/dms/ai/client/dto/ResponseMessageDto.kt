package dms.ai.client.dto

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/**
 * The answer message. LM Studio puts a reasoning model's thoughts in [reasoningContent]; with a JSON
 * schema it can put the whole answer there and leave [content] empty.
 */
@Serializable
data class ResponseMessageDto(
    val content: String? = null,
    @SerialName(value = "reasoning_content")
    val reasoningContent: String? = null,
)
