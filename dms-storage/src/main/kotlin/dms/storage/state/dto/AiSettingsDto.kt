package dms.storage.state.dto

import dms.application.port.state.model.AiSettings

/** The AI part of `settings.yaml`. Every field is optional, so older files keep loading. */
data class AiSettingsDto(
    val baseUrl: String = AiSettings.DEFAULT_BASE_URL,
    val model: String? = null,
    val preferVisionForPdfs: Boolean = false,
    val timeoutSeconds: Int = AiSettings.DEFAULT_TIMEOUT_SECONDS,
    val acceptSuggestedFileNames: Boolean = false,
)
