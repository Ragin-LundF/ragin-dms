package dms.ai.client.dto

import kotlinx.serialization.Serializable

/**
 * The JSON the model answers with, as `SuggestionSchema` describes it. Everything is optional: what
 * the model leaves out is simply not suggested.
 */
@Serializable
data class SuggestionDto(
    val title: String? = null,
    val category: String? = null,
    val tags: List<String> = emptyList(),
    val documentDate: String? = null,
    val correspondent: String? = null,
    val fileName: String? = null,
    val transcription: String? = null,
    val confidence: Double? = null,
)
