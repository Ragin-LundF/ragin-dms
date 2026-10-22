package dms.ai.prompt

import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.add
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonArray
import kotlinx.serialization.json.putJsonObject

/**
 * The JSON schema of the answer, sent as `response_format`, so the server constrains the model to
 * valid JSON. Strict mode needs every property listed as required; "not known" is `null`.
 */
object SuggestionSchema {

    private val NULLABLE_TEXT = listOf(
        "title",
        "category",
        "documentDate",
        "correspondent",
        "fileName",
        "transcription",
        "targetFolder",
        "targetReason",
    )

    val schema: JsonObject = buildJsonObject {
        put(key = "type", value = "object")
        put(key = "additionalProperties", value = false)
        putJsonObject(key = "properties") {
            NULLABLE_TEXT.forEach { name ->
                putJsonObject(key = name) {
                    putJsonArray(key = "type") {
                        add(value = "string")
                        add(value = "null")
                    }
                }
            }
            putJsonObject(key = "tags") {
                put(key = "type", value = "array")
                putJsonObject(key = "items") { put(key = "type", value = "string") }
            }
            putJsonObject(key = "pages") {
                put(key = "type", value = "array")
                putJsonObject(key = "items") { put(key = "type", value = "string") }
            }
            putJsonObject(key = "confidence") {
                putJsonArray(key = "type") {
                    add(value = "number")
                    add(value = "null")
                }
            }
            putJsonObject(key = "fields") {
                put(key = "type", value = "array")
                putJsonObject(key = "items") {
                    put(key = "type", value = "object")
                    put(key = "additionalProperties", value = false)
                    putJsonObject(key = "properties") {
                        putJsonObject(key = "key") { put(key = "type", value = "string") }
                        putJsonObject(key = "value") {
                            putJsonArray(key = "type") {
                                add(value = "string")
                                add(value = "null")
                            }
                        }
                    }
                    putJsonArray(key = "required") {
                        add(value = "key")
                        add(value = "value")
                    }
                }
            }
            putJsonObject(key = "targetFolderIsNew") {
                putJsonArray(key = "type") {
                    add(value = "boolean")
                    add(value = "null")
                }
            }
        }
        putJsonArray(key = "required") {
            (NULLABLE_TEXT + listOf("tags", "confidence", "targetFolderIsNew", "fields", "pages")).forEach {
                add(value = it)
            }
        }
    }
}
