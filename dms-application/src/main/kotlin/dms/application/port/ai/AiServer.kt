package dms.application.port.ai

import dms.application.port.ai.model.ModelList
import dms.application.port.state.model.AiSettings

/** The AI server itself: which models it offers, which doubles as the connection test. */
interface AiServer {

    suspend fun models(settings: AiSettings): ModelList
}
