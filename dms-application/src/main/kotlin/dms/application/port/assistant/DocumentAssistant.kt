package dms.application.port.assistant

import dms.application.port.assistant.model.AssistantAnswer
import dms.application.port.assistant.model.AssistantRequest
import dms.application.port.assistant.model.ChatTurn
import dms.domain.taxonomy.TagName

/**
 * Lets the AI read one document and answer about it: a summary, tags, or a question the user asked.
 * Answers are shown to the user; nothing is applied without the user's approval.
 */
interface DocumentAssistant {

    suspend fun summarize(request: AssistantRequest): AssistantAnswer

    /** Tags for the document, preferring [existingTags]; the answer is [AssistantAnswer.Tags]. */
    suspend fun suggestTags(request: AssistantRequest, existingTags: Set<TagName>): AssistantAnswer

    /** Answers the last question of [conversation], which alternates between the user and the AI. */
    suspend fun answer(request: AssistantRequest, conversation: List<ChatTurn>): AssistantAnswer
}
