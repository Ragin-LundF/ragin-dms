package dms.application.port.state.model

/**
 * Where the AI runs and how it is used, per device. The default is LM Studio's local server. [model]
 * `null` uses the first model the server has loaded. [preferVisionForPdfs] sends rendered pages of a
 * PDF even when it has a text layer, which helps with poor scanner OCR but is slower.
 */
data class AiSettings(
    val baseUrl: String = DEFAULT_BASE_URL,
    val model: String? = null,
    val preferVisionForPdfs: Boolean = false,
    val timeoutSeconds: Int = DEFAULT_TIMEOUT_SECONDS,
) {
    companion object {
        const val DEFAULT_BASE_URL: String = "http://localhost:1234/v1"
        const val DEFAULT_TIMEOUT_SECONDS: Int = 300
    }
}
