package dms.application.fake.ai

import dms.application.port.assistant.LibraryAssistant
import dms.application.port.assistant.model.AssistantAnswer
import dms.application.port.assistant.model.SourceChunkModel

/** Answers with [answer] and records the passages it was given. */
class FakeLibraryAssistant : LibraryAssistant {

    var answer: String = "See [1]."
    val sources: MutableList<List<SourceChunkModel>> = mutableListOf()

    override suspend fun answer(question: String, sources: List<SourceChunkModel>): AssistantAnswer {
        this.sources += sources
        return AssistantAnswer.Text(text = answer, model = "fake-chat")
    }
}
