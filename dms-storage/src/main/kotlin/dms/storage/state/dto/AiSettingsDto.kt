package dms.storage.state.dto

import dms.application.port.state.model.AiProvider
import dms.application.port.state.model.AiSettingsModel

/** The AI part of `settings.yaml`. Every field is optional, so older files keep loading. */
data class AiSettingsDto(
    val baseUrl: String = AiSettingsModel.DEFAULT_BASE_URL,
    val model: String? = null,
    val preferVisionForPdfs: Boolean = false,
    val timeoutSeconds: Int = AiSettingsModel.DEFAULT_TIMEOUT_SECONDS,
    val acceptSuggestedFileNames: Boolean = false,
    val embeddingModel: String? = null,
    val inputCharacters: Int = AiSettingsModel.DEFAULT_INPUT_CHARACTERS,
    val provider: String = AiProvider.LOCAL.name,
    val anthropicModel: String = AiSettingsModel.DEFAULT_ANTHROPIC_MODEL,
)
