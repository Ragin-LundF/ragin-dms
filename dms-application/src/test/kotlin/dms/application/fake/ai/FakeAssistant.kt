package dms.application.fake.ai

import dms.application.port.assistant.DocumentAssistant
import dms.application.port.assistant.model.AssistantAnswer
import dms.application.port.assistant.model.AssistantRequest
import dms.application.port.assistant.model.ChatTurn
import dms.domain.taxonomy.TagName

/** Answers with [summary], [tags] and [answer] and records what it was asked. */
class FakeAssistant : DocumentAssistant {

    var summary: AssistantAnswer = AssistantAnswer.Text(text = "A synthetic summary", model = MODEL)
    var tags: AssistantAnswer = AssistantAnswer.Tags(tags = listOf(TagName.of(raw = "Sample")), model = MODEL)
    var answer: AssistantAnswer = AssistantAnswer.Text(text = "A synthetic answer", model = MODEL)
    val requests: MutableList<AssistantRequest> = mutableListOf()
    val conversations: MutableList<List<ChatTurn>> = mutableListOf()

    override suspend fun summarize(request: AssistantRequest): AssistantAnswer {
        requests += request
        return summary
    }

    override suspend fun suggestTags(request: AssistantRequest, existingTags: Set<TagName>): AssistantAnswer {
        requests += request
        return tags
    }

    override suspend fun answer(request: AssistantRequest, conversation: List<ChatTurn>): AssistantAnswer {
        requests += request
        conversations += conversation
        return answer
    }

    companion object {
        const val MODEL: String = "test-model"
    }
}
