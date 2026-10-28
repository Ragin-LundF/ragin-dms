package dms.ai.prompt

import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.add
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonArray
import kotlinx.serialization.json.putJsonObject

/** The strict JSON schema of a page layout: the text lines and the graphics, each with its box. */
object RebuildLayoutSchema {

    private const val BOX_SIZE = 4

    private val box = buildJsonObject {
        put(key = "type", value = "array")
        putJsonObject(key = "items") { put(key = "type", value = "integer") }
        put(key = "minItems", value = BOX_SIZE)
        put(key = "maxItems", value = BOX_SIZE)
    }

    private val line = buildJsonObject {
        put(key = "type", value = "object")
        putJsonObject(key = "properties") {
            put(key = "bbox", element = box)
            putJsonObject(key = "text") { put(key = "type", value = "string") }
            putJsonObject(key = "bold") { put(key = "type", value = "boolean") }
            putJsonObject(key = "font") {
                put(key = "type", value = "string")
                putJsonArray(key = "enum") {
                    add(value = "sans")
                    add(value = "serif")
                    add(value = "mono")
                }
            }
            putJsonObject(key = "legible") { put(key = "type", value = "boolean") }
        }
        putJsonArray(key = "required") {
            listOf("bbox", "text", "bold", "font", "legible").forEach { add(value = it) }
        }
        put(key = "additionalProperties", value = false)
    }

    private val graphic = buildJsonObject {
        put(key = "type", value = "object")
        putJsonObject(key = "properties") {
            putJsonObject(key = "kind") { put(key = "type", value = "string") }
            put(key = "bbox", element = box)
        }
        putJsonArray(key = "required") {
            add(value = "kind")
            add(value = "bbox")
        }
        put(key = "additionalProperties", value = false)
    }

    val schema: JsonObject = buildJsonObject {
        put(key = "type", value = "object")
        putJsonObject(key = "properties") {
            putJsonObject(key = "lines") {
                put(key = "type", value = "array")
                put(key = "items", element = line)
            }
            putJsonObject(key = "graphics") {
                put(key = "type", value = "array")
                put(key = "items", element = graphic)
            }
        }
        putJsonArray(key = "required") {
            add(value = "lines")
            add(value = "graphics")
        }
        put(key = "additionalProperties", value = false)
    }
}
