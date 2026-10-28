package dms.ai.client.dto

import kotlinx.serialization.Serializable

/** One block of an Anthropic answer; only `text` blocks carry the answer, thinking blocks are skipped. */
@Serializable
data class ContentBlockDto(
    val type: String,
    val text: String? = null,
)
