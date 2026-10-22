package dms.application.port.assistant

import dms.application.port.assistant.model.AssistantAnswer
import dms.application.port.assistant.model.AssistantRequestModel
import dms.application.port.assistant.model.ChatTurnModel
import dms.domain.taxonomy.model.TagNameModel

/**
 * Lets the AI read one document and answer about it: a summary, tags, or a question the user asked.
 * Answers are shown to the user; nothing is applied without the user's approval.
 */
interface DocumentAssistant {

    suspend fun summarize(request: AssistantRequestModel): AssistantAnswer

    /** Tags for the document, preferring [existingTags]; the answer is [AssistantAnswer.Tags]. */
    suspend fun suggestTags(request: AssistantRequestModel, existingTags: Set<TagNameModel>): AssistantAnswer

    /** Answers the last question of [conversation], which alternates between the user and the AI. */
    suspend fun answer(request: AssistantRequestModel, conversation: List<ChatTurnModel>): AssistantAnswer
}
