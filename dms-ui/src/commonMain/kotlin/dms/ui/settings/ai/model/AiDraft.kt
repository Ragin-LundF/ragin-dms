package dms.ui.settings.ai.model

import dms.application.port.state.model.AiSettings

/** The AI settings as typed, before they are saved. A blank [model] uses the server's first loaded model. */
data class AiDraft(
    val baseUrl: String,
    val model: String,
    val timeoutSeconds: String,
    val preferVisionForPdfs: Boolean,
    val acceptSuggestedFileNames: Boolean,
) {

    /** The settings this draft describes, or `null` while a field is not valid. */
    fun toSettings(): AiSettings? {
        val timeout = timeoutSeconds.trim().toIntOrNull()?.takeIf { it > 0 }
        val url = baseUrl.trim().trimEnd('/')
        if (timeout == null || !(url.startsWith(prefix = "http://") || url.startsWith(prefix = "https://"))) {
            return null
        }
        return AiSettings(
            baseUrl = url,
            model = model.trim().ifEmpty { null },
            preferVisionForPdfs = preferVisionForPdfs,
            timeoutSeconds = timeout,
            acceptSuggestedFileNames = acceptSuggestedFileNames,
        )
    }

    companion object {

        fun of(settings: AiSettings): AiDraft {
            return AiDraft(
                baseUrl = settings.baseUrl,
                model = settings.model.orEmpty(),
                timeoutSeconds = settings.timeoutSeconds.toString(),
                preferVisionForPdfs = settings.preferVisionForPdfs,
                acceptSuggestedFileNames = settings.acceptSuggestedFileNames,
            )
        }
    }
}
