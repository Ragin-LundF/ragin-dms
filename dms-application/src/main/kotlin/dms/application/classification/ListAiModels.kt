package dms.application.classification

import dms.application.port.ai.AiServer
import dms.application.port.ai.model.ModelList
import dms.application.port.state.model.AiSettingsModel

/** Asks the AI server which models it offers; the settings use it as their connection test. */
class ListAiModels(private val server: AiServer) {

    suspend operator fun invoke(settings: AiSettingsModel): ModelList {
        return server.models(settings = settings)
    }
}
