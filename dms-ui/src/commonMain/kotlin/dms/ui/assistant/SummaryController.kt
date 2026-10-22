package dms.ui.assistant

import dms.application.assistant.model.AnswerOutcome
import dms.domain.document.model.DocumentModel
import dms.ui.assistant.model.SummaryStateModel
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

    private val mutableState = MutableStateFlow<SummaryStateModel?>(value = null)
    private var job: Job? = null

    val state: StateFlow<SummaryStateModel?> = mutableState.asStateFlow()

    fun open(document: DocumentModel) {
        job?.cancel()
        mutableState.value = SummaryStateModel(document = document)
        job = scope.launch {
            val outcome = services.ai.summarizeDocument(documentId = document.id)
            mutableState.value = SummaryStateModel(
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
