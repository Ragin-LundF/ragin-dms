package dms.application.assistant

import dms.application.assistant.model.AnswerOutcome
import dms.application.assistant.model.ReadingResult
import dms.application.port.assistant.DocumentAssistant
import dms.application.port.assistant.model.AssistantRequest
import dms.domain.document.DocumentId

/** Lets the AI summarize one document. Only shows the summary; nothing is stored. */
class SummarizeDocument(
    private val reading: DocumentReading,
    private val assistant: DocumentAssistant,
) {

    suspend operator fun invoke(documentId: DocumentId): AnswerOutcome {
        val read = when (val result = reading.read(documentId = documentId)) {
            is ReadingResult.Read -> result
            else -> return AssistantOutcomes.of(problem = result)
        }
        val request = AssistantRequest(fileName = read.document.path.fileName, content = read.content)
        return AssistantOutcomes.of(answer = assistant.summarize(request = request))
    }
}
