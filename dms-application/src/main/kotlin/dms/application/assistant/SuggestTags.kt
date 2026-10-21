package dms.application.assistant

import dms.application.assistant.model.ReadingResult
import dms.application.classification.model.SuggestOutcome
import dms.application.port.ai.model.ClassificationFailure
import dms.application.port.assistant.DocumentAssistant
import dms.application.port.assistant.model.AssistantAnswer
import dms.application.port.assistant.model.AssistantRequest
import dms.application.port.state.TaskStore
import dms.domain.classification.ClassificationSuggestion
import dms.domain.classification.SuggestedTag
import dms.domain.document.DocumentId

/**
 * Lets the AI suggest tags for one document. Tags the document already has are left out. A
 * suggestion with tags waits for the user's review like any other suggestion; one without new tags
 * is returned but not stored.
 */
class SuggestTags(
    private val reading: DocumentReading,
    private val assistant: DocumentAssistant,
    private val tasks: TaskStore,
) {

    suspend operator fun invoke(documentId: DocumentId): SuggestOutcome {
        val read = when (val result = reading.read(documentId = documentId)) {
            is ReadingResult.Read -> result
            ReadingResult.LibraryNotOpen -> return SuggestOutcome.LibraryNotOpen
            ReadingResult.NotFound -> return SuggestOutcome.NotFound
            is ReadingResult.NotReadable -> return SuggestOutcome.NotReadable(detail = result.detail)
        }
        val request = AssistantRequest(fileName = read.document.path.fileName, content = read.content)
        return when (val answer = assistant.suggestTags(request = request, existingTags = read.catalog.tags)) {
            is AssistantAnswer.Failed -> SuggestOutcome.Failed(reason = answer.reason, detail = answer.detail)
            is AssistantAnswer.Text -> SuggestOutcome.Failed(
                reason = ClassificationFailure.INVALID_RESPONSE,
                detail = answer.text.take(n = DETAIL_LENGTH),
            )

            is AssistantAnswer.Tags -> {
                val suggestion = ClassificationSuggestion(
                    documentId = documentId,
                    model = answer.model,
                    tags = answer.tags
                        .filter { it !in read.document.tags }
                        .distinct()
                        .map { SuggestedTag(name = it, isNew = it !in read.catalog.tags) },
                )
                if (suggestion.tags.isNotEmpty()) {
                    tasks.saveSuggestion(suggestion = suggestion)
                }
                SuggestOutcome.Suggested(suggestion = suggestion)
            }
        }
    }

    private companion object {
        const val DETAIL_LENGTH = 300
    }
}
