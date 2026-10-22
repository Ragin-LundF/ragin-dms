package dms.application.port.assistant

import dms.application.port.assistant.model.AssistantAnswer
import dms.application.port.assistant.model.SourceChunkModel

/**
 * Answers a question about the whole library from the passages found for it, citing them as `[n]`,
 * and saying so plainly when they do not hold the answer.
 */
interface LibraryAssistant {

    suspend fun answer(question: String, sources: List<SourceChunkModel>): AssistantAnswer
}
