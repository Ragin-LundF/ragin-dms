package dms.application.assistant

import dms.application.assistant.model.AnswerOutcome
import dms.application.assistant.model.ReadingResult
import dms.application.library.CatalogStore
import dms.application.port.assistant.DocumentAssistant
import dms.application.port.assistant.model.AssistantRequestModel
import dms.application.port.assistant.model.ChatTurnModel
import dms.domain.document.model.DocumentIdModel

/**
 * Answers the user's questions about one document. The [conversation] so far goes with every
 * question, so follow-up questions work. The document is read once and kept while it is unchanged,
 * because rendering pages for every question would be slow.
 */
class AskAboutDocument(
    private val store: CatalogStore,
    private val reading: DocumentReading,
    private val assistant: DocumentAssistant,
) {

    private var cached: ReadingResult.Read? = null

    suspend operator fun invoke(documentId: DocumentIdModel, conversation: List<ChatTurnModel>): AnswerOutcome {
        val read = cachedRead(documentId = documentId)
            ?: when (val result = reading.read(documentId = documentId)) {
                is ReadingResult.Read -> result.also { cached = it }
                else -> return AssistantOutcomes.of(problem = result)
            }
        val request = AssistantRequestModel(fileName = read.document.path.fileName, content = read.content)
        return AssistantOutcomes.of(answer = assistant.answer(request = request, conversation = conversation))
    }

    /** The cached document, while the catalog still describes it the same way. */
    private suspend fun cachedRead(documentId: DocumentIdModel): ReadingResult.Read? {
        val current = store.current()?.catalog?.documents?.get(documentId)
        return cached?.takeIf { it.document.id == documentId && it.document == current }
    }
}
