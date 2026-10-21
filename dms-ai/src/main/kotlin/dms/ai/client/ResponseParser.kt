package dms.ai.client

import dms.ai.client.dto.SuggestionDto
import kotlinx.serialization.json.Json

/**
 * Reads the model's JSON. Servers that ignore the schema sometimes wrap it in a Markdown code fence or
 * add a sentence around it, so the outermost object is taken.
 */
object ResponseParser {

    private val json = Json {
        ignoreUnknownKeys = true
        isLenient = true
        coerceInputValues = true
    }

    fun parse(content: String): SuggestionDto? {
        val start = content.indexOf(char = '{')
        val end = content.lastIndexOf(char = '}')
        if (start < 0 || end <= start) {
            return null
        }
        return try {
            json.decodeFromString<SuggestionDto>(string = content.substring(startIndex = start, endIndex = end + 1))
        } catch (ignored: IllegalArgumentException) {
            // Also every SerializationException.
            null
        }
    }
}
