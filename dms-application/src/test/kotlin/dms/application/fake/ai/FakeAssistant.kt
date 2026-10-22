package dms.application.fake.ai

import dms.application.port.assistant.DocumentAssistant
import dms.application.port.assistant.model.AssistantAnswer
import dms.application.port.assistant.model.AssistantRequestModel
import dms.application.port.assistant.model.ChatTurnModel
import dms.domain.taxonomy.model.TagNameModel

/** Answers with [summary], [tags] and [answer] and records what it was asked. */
class FakeAssistant : DocumentAssistant {

    var summary: AssistantAnswer = AssistantAnswer.Text(text = "A synthetic summary", model = MODEL)
    var tags: AssistantAnswer = AssistantAnswer.Tags(tags = listOf(TagNameModel.of(raw = "Sample")), model = MODEL)
    var answer: AssistantAnswer = AssistantAnswer.Text(text = "A synthetic answer", model = MODEL)
    val requests: MutableList<AssistantRequestModel> = mutableListOf()
    val conversations: MutableList<List<ChatTurnModel>> = mutableListOf()

    override suspend fun summarize(request: AssistantRequestModel): AssistantAnswer {
        requests += request
        return summary
    }

    override suspend fun suggestTags(request: AssistantRequestModel, existingTags: Set<TagNameModel>): AssistantAnswer {
        requests += request
        return tags
    }

    override suspend fun answer(request: AssistantRequestModel, conversation: List<ChatTurnModel>): AssistantAnswer {
        requests += request
        conversations += conversation
        return answer
    }

    companion object {
        const val MODEL: String = "test-model"
    }
}
