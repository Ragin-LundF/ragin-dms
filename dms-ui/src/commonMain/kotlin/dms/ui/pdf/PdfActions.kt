package dms.ui.pdf

import dms.application.pdf.model.SearchableOutcome
import dms.domain.document.DocumentKind
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.ui.components.dialog.DialogController
import dms.ui.components.model.ConfirmationRequestModel
import dms.ui.document.details.model.LinkCandidateModel
import dms.ui.pdf.model.MergeStateModel
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.UiMessageModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import java.util.Locale

/**
 * Making a scan searchable and merging PDFs, from the quickLookViewModel. "Precise OCR" is offered while the
 * OCR program is installed on this device.
 */
class PdfActions(
    private val services: AppServices,
    private val session: LibrarySession,
    private val dialogs: DialogController,
    scope: CoroutineScope,
) {

    private val ocrState = MutableStateFlow(value = false)
    private val mergeState = MutableStateFlow<MergeStateModel?>(value = null)

    /** Whether the OCR program is installed. */
    val ocrAvailable: StateFlow<Boolean> = ocrState.asStateFlow()
    val merge: StateFlow<MergeStateModel?> = mergeState.asStateFlow()

    init {
        scope.launch { ocrState.value = services.pdf.ocr.available() }
    }

    fun makeSearchable(document: DocumentModel, precise: Boolean) {
        val name = document.path.fileName
        session.perform(label = "Making $name searchable") {
            val outcome = services.pdf.makeSearchable(documentId = document.id, precise = precise)
            session.report(message = messageOf(outcome = outcome, name = name))
        }
    }

    fun startMerge(document: DocumentModel) {
        mergeState.value = MergeStateModel(
            chosen = listOf(candidateOf(document = document)),
            name = "${document.path.fileName.substringBeforeLast(delimiter = '.')} (merged)",
        )
        search(query = "")
    }

    fun search(query: String) {
        val current = mergeState.value ?: return
        val wanted = query.trim().lowercase(Locale.ROOT)
        val chosen = current.chosen.map { it.documentId }.toSet()
        val matches = services.library.catalog.value?.documents?.values.orEmpty()
            .filter { it.kind == DocumentKind.PDF && it.id !in chosen }
            .filter { wanted.isEmpty() || textOf(document = it).contains(other = wanted) }
            .sortedByDescending { it.addedAt }
            .take(n = MATCHES)
            .map { candidateOf(document = it) }
        mergeState.value = current.copy(query = query, matches = matches)
    }

    fun add(documentId: DocumentIdModel) {
        val candidate = mergeState.value?.matches?.firstOrNull { it.documentId == documentId } ?: return
        mergeState.update { it?.copy(chosen = it.chosen + candidate) }
        search(query = mergeState.value?.query.orEmpty())
    }

    fun remove(documentId: DocumentIdModel) {
        mergeState.update { state ->
            state?.copy(chosen = state.chosen.filterNot { it.documentId == documentId }.ifEmpty { state.chosen })
        }
    }

    fun setName(name: String) {
        mergeState.update { it?.copy(name = name) }
    }

    fun setTrashSources(trash: Boolean) {
        mergeState.update { it?.copy(trashSources = trash) }
    }

    fun cancelMerge() {
        mergeState.value = null
    }

    fun confirmMerge() {
        val state = mergeState.value ?: return
        if (state.chosen.size < 2 || state.name.isBlank()) {
            return
        }
        if (state.trashSources) {
            dialogs.ask(
                request = ConfirmationRequestModel(
                    title = "Merge ${state.chosen.size} PDFs",
                    message = "The merged file is created first; then the sources go to the trash in every folder.",
                    affected = state.chosen.map { it.path },
                    confirmLabel = "Merge and move to trash",
                ),
            ) { runMerge(state = state) }
        } else {
            runMerge(state = state)
        }
    }

    private fun runMerge(state: MergeStateModel) {
        val started = session.perform(label = "Merging ${state.chosen.size} PDFs") {
            val outcome = services.pdf.mergeDocuments(
                documentIds = state.chosen.map { it.documentId },
                fileName = state.name.trim(),
                trashSources = state.trashSources,
            )
            session.report(message = OutcomeMessages.of(outcome = outcome, done = "Merged into ${state.name.trim()}"))
        }
        if (started) {
            mergeState.value = null
        }
    }

    private fun messageOf(outcome: SearchableOutcome, name: String): UiMessageModel {
        return when (outcome) {
            is SearchableOutcome.Made -> OutcomeMessages.of(outcome = outcome.outcome, done = "$name is searchable now")
            SearchableOutcome.AlreadySearchable -> UiMessageModel(text = "$name has a text layer already")
            SearchableOutcome.NoTranscription -> UiMessageModel(
                text = "$name has no transcription yet: classify it with AI first, or use precise OCR",
                isError = true,
            )

            SearchableOutcome.OcrUnavailable -> UiMessageModel(
                text = "ocrmypdf is not installed on this device",
                isError = true,
            )
            SearchableOutcome.NotPdf -> UiMessageModel(text = "Only PDF files can be made searchable", isError = true)
            is SearchableOutcome.Failed -> OutcomeMessages.of(outcome = outcome.outcome, done = "")
        }
    }

    private fun textOf(document: DocumentModel): String {
        return listOfNotNull(
            document.path.value,
            document.metadata.title,
        ).joinToString(separator = " ").lowercase(Locale.ROOT)
    }

    private fun candidateOf(document: DocumentModel): LinkCandidateModel {
        return LinkCandidateModel(
            documentId = document.id,
            name = document.metadata.title ?: document.path.fileName,
            path = document.path.value,
        )
    }

    private companion object {
        const val MATCHES = 30
    }
}
