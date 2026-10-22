package dms.ui.settings.ai.model

import dms.application.port.state.model.AiSettingsModel

/**
 * The AI settings as typed, before they are saved. A blank [model] uses the server's first loaded
 * model, a blank [embeddingModel] the server's first embedding model.
 */
data class AiDraftModel(
    val baseUrl: String,
    val model: String,
    val timeoutSeconds: String,
    val preferVisionForPdfs: Boolean,
    val acceptSuggestedFileNames: Boolean,
    val embeddingModel: String = "",
) {

    /** The settings this draft describes, or `null` while a field is not valid. */
    fun toSettings(): AiSettingsModel? {
        val timeout = timeoutSeconds.trim().toIntOrNull()?.takeIf { it > 0 }
        val url = baseUrl.trim().trimEnd('/')
        if (timeout == null || !(url.startsWith(prefix = "http://") || url.startsWith(prefix = "https://"))) {
            return null
        }
        return AiSettingsModel(
            baseUrl = url,
            model = model.trim().ifEmpty { null },
            preferVisionForPdfs = preferVisionForPdfs,
            timeoutSeconds = timeout,
            acceptSuggestedFileNames = acceptSuggestedFileNames,
            embeddingModel = embeddingModel.trim().ifEmpty { null },
        )
    }

    companion object {

        fun of(settings: AiSettingsModel): AiDraftModel {
            return AiDraftModel(
                baseUrl = settings.baseUrl,
                model = settings.model.orEmpty(),
                timeoutSeconds = settings.timeoutSeconds.toString(),
                preferVisionForPdfs = settings.preferVisionForPdfs,
                acceptSuggestedFileNames = settings.acceptSuggestedFileNames,
                embeddingModel = settings.embeddingModel.orEmpty(),
            )
        }
    }
}
