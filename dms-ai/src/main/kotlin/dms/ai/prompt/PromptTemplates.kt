package dms.ai.prompt

/**
 * The prompt templates in `src/main/resources/prompts`. A template names its values as `{{name}}`;
 * values are filled in the given order, so put the document text last.
 */
object PromptTemplates {

    private val cache = mutableMapOf<String, String>()

    fun fill(name: String, values: Map<String, String>): String {
        return values.entries.fold(initial = template(name = name)) { text, (key, value) ->
            text.replace("{{$key}}", value)
        }
    }

    @Synchronized
    private fun template(name: String): String {
        return cache.getOrPut(key = name) {
            val stream = checkNotNull(value = PromptTemplates::class.java.getResourceAsStream("/prompts/$name")) {
                "Missing prompt resource $name"
            }
            stream.use { it.readBytes().decodeToString() }.trim()
        }
    }
}
