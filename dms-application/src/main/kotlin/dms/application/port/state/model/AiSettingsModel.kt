package dms.application.port.state.model

/**
 * Where the AI runs and how it is used, per device. The default is LM Studio's local server. [model]
 * `null` uses the first model the server has loaded. [preferVisionForPdfs] sends rendered pages of a
 * PDF even when it has a text layer, which helps with poor scanner OCR but is slower.
 * [acceptSuggestedFileNames] checks the suggested file name in a review, so approving renames the file;
 * otherwise the user checks it when the name is better. [embeddingModel] makes the vectors of the
 * library's index; `null` uses the first embedding model the server offers. [inputCharacters] is how much
 * of a document's text is read and sent to the AI; small local models need less, large ones take more.
 */
data class AiSettingsModel(
    val baseUrl: String = DEFAULT_BASE_URL,
    val model: String? = null,
    val preferVisionForPdfs: Boolean = false,
    val timeoutSeconds: Int = DEFAULT_TIMEOUT_SECONDS,
    val acceptSuggestedFileNames: Boolean = false,
    val embeddingModel: String? = null,
    val inputCharacters: Int = DEFAULT_INPUT_CHARACTERS,
) {
    init {
        require(value = inputCharacters in INPUT_CHARACTERS_RANGE) { "Input budget must be in $INPUT_CHARACTERS_RANGE" }
    }

    companion object {
        const val DEFAULT_INPUT_CHARACTERS: Int = 12_000
        val INPUT_CHARACTERS_RANGE: IntRange = 1_000..200_000
        const val DEFAULT_BASE_URL: String = "http://localhost:1234/v1"
        const val DEFAULT_TIMEOUT_SECONDS: Int = 300
    }
}
