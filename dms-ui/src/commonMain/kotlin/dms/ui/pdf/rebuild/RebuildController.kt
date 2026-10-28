package dms.ui.pdf.rebuild

import dms.application.common.OperationOutcome
import dms.application.pdf.rebuild.model.RebuildChoice
import dms.application.pdf.rebuild.model.RebuildOutcome
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.ui.PlatformActions
import dms.ui.components.dialog.DialogController
import dms.ui.components.model.ConfirmationRequestModel
import dms.ui.pdf.model.CompareView
import dms.ui.pdf.model.RebuildCompareStateModel
import dms.ui.service.AppServices
import dms.ui.session.LibrarySession
import dms.ui.session.OutcomeMessages
import dms.ui.session.model.OperationLane
import dms.ui.session.model.OperationProgressModel
import dms.ui.session.model.UiMessageModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/**
 * "Convert to real PDF": runs the rebuild in the AI lane with progress per page, then shows the scan and
 * its rebuild side by side until the user keeps the original, uses the rebuild (after a confirmation,
 * since the scan goes to the trash) or keeps both. Only these three buttons close the comparison, so a
 * stray click never throws away a run of many minutes; one comparison is open at a time.
 */
class RebuildController(
    private val services: AppServices,
    private val session: LibrarySession,
    private val platform: PlatformActions,
    private val dialogs: DialogController,
    private val scope: CoroutineScope,
) {

    private val rebuildable = MutableStateFlow<Set<DocumentIdModel>>(value = emptySet())
    private val compareState = MutableStateFlow<RebuildCompareStateModel?>(value = null)

    /** The documents that were checked and can be converted. */
    val available: StateFlow<Set<DocumentIdModel>> = rebuildable.asStateFlow()
    val compare: StateFlow<RebuildCompareStateModel?> = compareState.asStateFlow()

    /** Why the AI cannot take a conversion now, or `null`. */
    val aiBusyReason: StateFlow<String?> = session.state
        .map { state -> state.aiBusy?.let { OutcomeMessages.waitFor(operation = it) } }
        .stateIn(scope = scope, started = SharingStarted.Eagerly, initialValue = null)

    /** Finds out in the background whether [document] can be converted; [available] says the result. */
    fun check(document: DocumentModel) {
        scope.launch {
            val possible = services.pdf.rebuildAvailability(documentId = document.id)
            rebuildable.update { if (possible) it + document.id else it - document.id }
        }
    }

    fun start(document: DocumentModel) {
        if (compareState.value != null) {
            session.report(message = UiMessageModel(text = OPEN_COMPARISON, isError = true))
            return
        }
        val name = document.path.fileName
        session.perform(label = "Converting $name to a real PDF", lane = OperationLane.AI, cancellable = true) {
            val outcome = services.pdf.rebuildPdf(documentId = document.id) { page, pages ->
                session.progress(progress = progressOf(page = page, pages = pages), lane = OperationLane.AI)
            }
            session.report(message = RebuildTexts.messageOf(outcome = outcome, name = name))
            if (outcome is RebuildOutcome.Rebuilt) {
                compareState.value = RebuildCompareStateModel(
                    documentId = document.id,
                    path = document.path,
                    rebuilt = outcome.rebuilt,
                    originalSize = outcome.originalSize,
                )
                load(page = 0)
            }
        }
    }

    fun showPage(page: Int) {
        val state = compareState.value ?: return
        if (page !in 0 until state.pages || page == state.page) return
        compareState.value = state.copy(page = page, original = null, rebuiltImage = null)
        load(page = page)
    }

    /** Shows one page in place and turns it between the scan and the rebuild. */
    fun flip() {
        compareState.update { state ->
            state?.copy(view = if (state.view == CompareView.REBUILT) CompareView.ORIGINAL else CompareView.REBUILT)
        }
    }

    fun sideBySide() {
        compareState.update { it?.copy(view = CompareView.SIDE_BY_SIDE) }
    }

    fun toggleOverlay() {
        compareState.update { it?.copy(overlay = !it.overlay) }
    }

    fun choose(choice: RebuildChoice) {
        val state = compareState.value ?: return
        if (choice != RebuildChoice.REPLACE_WITH_REBUILD) {
            keep(state = state, choice = choice)
            return
        }
        dialogs.ask(
            request = ConfirmationRequestModel(
                title = "Use the rebuilt PDF",
                message = "The scan moves to the trash, and the rebuilt PDF takes its place in every folder.",
                affected = listOf(state.path.value),
                confirmLabel = "Use rebuilt PDF",
            ),
        ) { keep(state = state, choice = choice) }
    }

    private fun keep(state: RebuildCompareStateModel, choice: RebuildChoice) {
        val name = state.path.fileName
        val started = session.perform(label = labelOf(choice = choice, name = name)) {
            val outcome = services.pdf.keepRebuild(
                documentId = state.documentId,
                rebuilt = state.rebuilt,
                choice = choice,
            )
            session.report(message = messageOf(outcome = outcome, choice = choice, name = name))
            // Otherwise the rebuild stays, so the user can choose again.
            if (outcome is OperationOutcome.Done || outcome in GONE) {
                compareState.value = null
            }
        }
        if (!started) {
            val running = session.state.value.busy.orEmpty()
            val wait = OutcomeMessages.waitFor(operation = running)
            session.report(message = UiMessageModel(text = wait, isError = true))
        }
    }

    private fun messageOf(outcome: OperationOutcome, choice: RebuildChoice, name: String): UiMessageModel {
        if (outcome == OperationOutcome.SourceChanged) {
            val text = "$name changed after it was converted: convert it again, or keep both"
            return UiMessageModel(text = text, isError = true)
        }
        return OutcomeMessages.of(outcome = outcome, done = doneOf(choice = choice, name = name))
    }

    private fun progressOf(page: Int, pages: Int): OperationProgressModel {
        val step = if (page < pages) "Page ${page + 1} of $pages · a few minutes per page" else "Writing the PDF"
        return OperationProgressModel(step = step, done = page, total = pages)
    }

    private fun load(page: Int) {
        val state = compareState.value ?: return
        val primary = session.state.value.folders?.primary ?: return
        scope.launch {
            val original = services.pdf.editor.renderPage(
                folder = primary,
                path = state.path,
                page = page,
                maxSide = SIDE,
            )
            val rebuilt = services.pdf.rebuilder.renderPage(rebuilt = state.rebuilt, page = page, maxSide = SIDE)
            compareState.update { current ->
                // The user may have turned the page or closed the comparison meanwhile.
                if (current == null || current.rebuilt != state.rebuilt || current.page != page) return@update current
                current.copy(
                    original = original?.let { platform.decodeImage(bytes = it.bytes) },
                    rebuiltImage = rebuilt?.let { platform.decodeImage(bytes = it.bytes) },
                )
            }
        }
    }

    private fun labelOf(choice: RebuildChoice, name: String): String {
        return when (choice) {
            RebuildChoice.KEEP_ORIGINAL -> "Keeping the original of $name"
            RebuildChoice.REPLACE_WITH_REBUILD -> "Replacing $name with its rebuild"
            RebuildChoice.KEEP_BOTH -> "Adding the rebuild of $name"
        }
    }

    private fun doneOf(choice: RebuildChoice, name: String): String {
        return when (choice) {
            RebuildChoice.KEEP_ORIGINAL -> "Kept the original of $name"
            RebuildChoice.REPLACE_WITH_REBUILD -> "$name is a real PDF now; the scan is in the trash"
            RebuildChoice.KEEP_BOTH -> "Added the rebuild of $name next to it"
        }
    }

    private companion object {
        /** About 100 dpi for an A4 page. */
        const val SIDE = 1170
        const val OPEN_COMPARISON = "Choose what to keep in the open comparison first"

        /** Outcomes after which the rebuild was dropped, since it has nowhere to go any more. */
        val GONE = setOf(OperationOutcome.NotFound, OperationOutcome.LibraryNotOpen)
    }
}
