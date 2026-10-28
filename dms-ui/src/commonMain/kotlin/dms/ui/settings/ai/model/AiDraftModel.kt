package dms.ui.settings.ai.model

import dms.application.port.state.model.AiProvider
import dms.application.port.state.model.AiSettingsModel

/**
 * The AI settings as typed, before they are saved. A blank [model] uses the server's first loaded
 * model, a blank [embeddingModel] the server's first embedding model. [inputCharacters] must lie in
 * [AiSettingsModel.INPUT_CHARACTERS_RANGE]. The local server and Anthropic are both kept; [provider] picks
 * which one answers.
 */
data class AiDraftModel(
    val enabled: Boolean = true,
    val baseUrl: String,
    val model: String,
    val timeoutSeconds: String,
    val preferVisionForPdfs: Boolean,
    val acceptSuggestedFileNames: Boolean,
    val embeddingModel: String = "",
    val inputCharacters: String = AiSettingsModel.DEFAULT_INPUT_CHARACTERS.toString(),
    val provider: AiProvider = AiProvider.LOCAL,
    val anthropicModel: String = AiSettingsModel.DEFAULT_ANTHROPIC_MODEL,
) {

    /** The address is only checked for Local AI; Anthropic's address is fixed and its field hidden. */
    fun addressValid(): Boolean {
        val url = baseUrl.trim()
        return provider != AiProvider.LOCAL || url.startsWith(prefix = "http://") || url.startsWith(prefix = "https://")
    }

    fun timeoutValid(): Boolean {
        return (timeoutSeconds.trim().toIntOrNull() ?: 0) > 0
    }

    fun inputValid(): Boolean {
        return inputCharacters.trim().toIntOrNull() in AiSettingsModel.INPUT_CHARACTERS_RANGE
    }

    /** What to fix before the draft can be tested or saved, naming the field; `null` when it is valid. */
    fun problem(): String? {
        val range = AiSettingsModel.INPUT_CHARACTERS_RANGE
        return when {
            !addressValid() -> "Enter a server address starting with http:// or https://"
            !timeoutValid() -> "Enter a timeout above 0 seconds"
            !inputValid() -> "Enter a text size from ${range.first} to ${range.last} characters"
            else -> null
        }
    }

    /** The settings this draft describes, or `null` while a field is not valid. */
    fun toSettings(): AiSettingsModel? {
        if (problem() != null) {
            return null
        }
        return AiSettingsModel(
            enabled = enabled,
            baseUrl = baseUrl.trim().trimEnd('/'),
            model = model.trim().ifEmpty { null },
            preferVisionForPdfs = preferVisionForPdfs,
            timeoutSeconds = timeoutSeconds.trim().toInt(),
            acceptSuggestedFileNames = acceptSuggestedFileNames,
            embeddingModel = embeddingModel.trim().ifEmpty { null },
            inputCharacters = inputCharacters.trim().toInt(),
            provider = provider,
            anthropicModel = anthropicModel.trim().ifEmpty { AiSettingsModel.DEFAULT_ANTHROPIC_MODEL },
        )
    }

    companion object {

        fun of(settings: AiSettingsModel): AiDraftModel {
            return AiDraftModel(
                enabled = settings.enabled,
                baseUrl = settings.baseUrl,
                model = settings.model.orEmpty(),
                timeoutSeconds = settings.timeoutSeconds.toString(),
                preferVisionForPdfs = settings.preferVisionForPdfs,
                acceptSuggestedFileNames = settings.acceptSuggestedFileNames,
                embeddingModel = settings.embeddingModel.orEmpty(),
                inputCharacters = settings.inputCharacters.toString(),
                provider = settings.provider,
                anthropicModel = settings.anthropicModel,
            )
        }
    }
}
