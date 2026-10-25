package dms.ui.review

import dms.application.common.OperationOutcome
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.ui.review.model.ReviewDraftModel
import dms.ui.review.model.ReviewStateModel
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.UiMessageModel
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

    private val mutableState = MutableStateFlow<ReviewStateModel?>(value = null)

    val state: StateFlow<ReviewStateModel?> = mutableState.asStateFlow()

    fun open(suggestion: ClassificationSuggestionModel) {
        val catalog = services.library.catalog.value
        val document = catalog?.documents?.get(suggestion.documentId)
        if (catalog == null || document == null) {
            session.report(message = UiMessageModel(text = "The document of this suggestion is gone", isError = true))
            return
        }
        mutableState.value = ReviewStateModel(
            suggestion = suggestion,
            documentName = document.path.value,
            current = ReviewDrafts.currentOf(document = document, catalog = catalog),
            draft = ReviewDrafts.draftOf(
                suggestion = suggestion,
                catalog = catalog,
                acceptFileName = session.state.value.preferences.ai.acceptSuggestedFileNames,
            ),
            categoryPaths = ReviewDrafts.categoryPathsOf(catalog = catalog),
            folderPaths = ReviewDrafts.folderPathsOf(catalog = catalog),
            fieldLabels = catalog.settings.fieldDefinitions.associate { it.key to it.label },
        )
    }

    fun update(draft: ReviewDraftModel) {
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
            } else if (outcome.catalog.settings.layout.embedMetadata) {
                val embedded = services.pdf.embedPdfMetadata(documentId = documentId)
                if (embedded !is OperationOutcome.Done) {
                    session.report(message = OutcomeMessages.of(outcome = embedded, done = ""))
                }
            }
        }
        if (started) {
            close()
        } else {
            val running = session.state.value.busy.orEmpty()
            mutableState.value = review.copy(problem = OutcomeMessages.waitFor(operation = running))
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
