package dms.ui.review

import dms.application.common.OperationOutcome
import dms.domain.classification.ClassificationSuggestion
import dms.ui.review.model.ReviewDraft
import dms.ui.review.model.ReviewState
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.UiMessage
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update

/**
 * The review of one suggestion: a view model opens it, the user edits and approves or rejects it,
 * and a `ReviewDialog` shows it. Nothing changes before the user approves.
 */
class ReviewController(
    private val services: AppServices,
    private val session: LibrarySession,
) {

    private val mutableState = MutableStateFlow<ReviewState?>(value = null)

    val state: StateFlow<ReviewState?> = mutableState.asStateFlow()

    fun open(suggestion: ClassificationSuggestion) {
        val catalog = services.library.catalog.value
        val document = catalog?.documents?.get(suggestion.documentId)
        if (catalog == null || document == null) {
            session.report(message = UiMessage(text = "The document of this suggestion is gone", isError = true))
            return
        }
        mutableState.value = ReviewState(
            suggestion = suggestion,
            documentName = document.path.value,
            current = ReviewDrafts.currentOf(document = document, catalog = catalog),
            draft = ReviewDrafts.draftOf(suggestion = suggestion, catalog = catalog),
        )
    }

    fun update(draft: ReviewDraft) {
        mutableState.update { it?.copy(draft = draft, problem = null) }
    }

    fun approve() {
        val review = mutableState.value ?: return
        val catalog = services.library.catalog.value ?: return
        val approval = ReviewDrafts.approvalOf(draft = review.draft, current = review.current, catalog = catalog)
            .getOrElse { failure ->
                mutableState.value = review.copy(problem = failure.message)
                return
            }
        val documentId = review.suggestion.documentId
        val started = session.perform(label = "Applying the suggestion for ${review.documentName}") {
            val outcome = services.ai.approveSuggestion(documentId = documentId, approval = approval)
            session.report(message = OutcomeMessages.of(outcome = outcome, done = "Filed ${review.documentName}"))
            if (outcome !is OperationOutcome.Done) {
                session.log.error(text = "Approval refused for ${review.documentName}")
            }
        }
        if (started) {
            close()
        } else {
            mutableState.value = review.copy(problem = "Wait until \"${session.state.value.busy}\" finishes")
        }
    }

    fun reject() {
        val review = mutableState.value ?: return
        val started = session.perform(label = "Rejecting the suggestion for ${review.documentName}") {
            services.ai.rejectSuggestion(documentId = review.suggestion.documentId)
        }
        if (started) {
            close()
        }
    }

    fun close() {
        mutableState.value = null
    }
}
