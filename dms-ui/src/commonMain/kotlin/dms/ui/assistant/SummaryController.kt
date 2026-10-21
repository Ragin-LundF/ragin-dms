package dms.ui.assistant

import dms.application.assistant.model.AnswerOutcome
import dms.domain.document.Document
import dms.ui.assistant.model.SummaryState
import dms.ui.review.AiMessages
import dms.ui.service.AppServices
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

/**
 * The AI's summary of one document. It only reads, so it runs beside whatever the library does;
 * closing the dialog stops the AI.
 */
class SummaryController(
    private val services: AppServices,
    private val scope: CoroutineScope,
) {

    private val mutableState = MutableStateFlow<SummaryState?>(value = null)
    private var job: Job? = null

    val state: StateFlow<SummaryState?> = mutableState.asStateFlow()

    fun open(document: Document) {
        job?.cancel()
        mutableState.value = SummaryState(document = document)
        job = scope.launch {
            val outcome = services.ai.summarizeDocument(documentId = document.id)
            mutableState.value = SummaryState(
                document = document,
                working = false,
                text = (outcome as? AnswerOutcome.Answered)?.text,
                model = (outcome as? AnswerOutcome.Answered)?.model,
                problem = AiMessages.problemOf(outcome = outcome, name = document.path.fileName),
            )
        }
    }

    fun close() {
        job?.cancel()
        mutableState.value = null
    }
}
