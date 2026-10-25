package dms.application.fake.ai

import dms.application.port.ai.AiServer
import dms.application.port.ai.model.ModelList
import dms.application.port.state.model.AiSettingsModel

/** Answers every request with [answer] and records the settings it was asked with. */
class FakeAiServer : AiServer {

    var answer: ModelList = ModelList.Available(models = emptyList())
    val requests: MutableList<AiSettingsModel> = mutableListOf()

    override suspend fun models(settings: AiSettingsModel): ModelList {
        requests += settings
        return answer
    }
}
