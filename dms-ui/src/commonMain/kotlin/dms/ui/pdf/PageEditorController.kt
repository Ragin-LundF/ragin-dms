package dms.ui.pdf

import dms.application.common.OperationOutcome
import dms.application.port.pdf.model.PageChangeModel
import dms.application.port.storage.model.StorageResult
import dms.domain.document.model.DocumentModel
import dms.ui.PlatformActions
import dms.ui.components.dialog.DialogController
import dms.ui.components.model.ConfirmationRequestModel
import dms.ui.pdf.model.PageEditorStateModel
import dms.ui.pdf.model.PageItemModel
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

/**
 * The page editor of a PDF: turn, remove and move pages, and mark where the document splits. Applying
 * writes a new version of the file (the old one goes to the trash in every folder) and then splits it,
 * if marked. The quickLookViewModel's view model owns it.
 */
class PageEditorController(
    private val services: AppServices,
    private val session: LibrarySession,
    private val platform: PlatformActions,
    private val dialogs: DialogController,
    private val scope: CoroutineScope,
) {

    private val mutableState = MutableStateFlow<PageEditorStateModel?>(value = null)

    val state: StateFlow<PageEditorStateModel?> = mutableState.asStateFlow()

    fun open(document: DocumentModel) {
        mutableState.value = PageEditorStateModel(documentId = document.id, name = document.path.fileName)
        scope.launch {
            val primary = session.state.value.folders?.primary
            if (primary == null) {
                mutableState.value = null
                session.report(message = UiMessageModel(text = "No library is open", isError = true))
                return@launch
            }
            val counted = services.pdf.editor.pageCount(folder = primary, path = document.path)
            val count = (counted as? StorageResult.Success)?.value ?: 0
            val pages = List(size = count) { index -> PageItemModel(source = index) }
            mutableState.update { it?.copy(pages = pages, loading = false) }
            repeat(times = count) { page ->
                val image = services.pdf.editor.renderPage(
                    folder = primary,
                    path = document.path,
                    page = page,
                    maxSide = THUMBNAIL,
                )
                val bitmap = image?.let { platform.decodeImage(bytes = it.bytes) } ?: return@repeat
                mutableState.update { it?.copy(thumbnails = it.thumbnails + (page to bitmap)) }
            }
        }
    }

    fun rotate(position: Int, degrees: Int) {
        change(position = position) { it.copy(rotation = Math.floorMod(it.rotation + degrees, FULL_TURN)) }
    }

    fun toggleDelete(position: Int) {
        change(position = position) { it.copy(deleted = !it.deleted) }
    }

    fun toggleSplit(position: Int) {
        change(position = position) { it.copy(splitAfter = !it.splitAfter) }
    }

    /** Moves the page at [position] one place to the left (−1) or the right (+1). */
    fun move(position: Int, by: Int) {
        mutableState.update { current ->
            val pages = current?.pages ?: return@update current
            val target = position + by
            if (target !in pages.indices) {
                return@update current
            }
            current.copy(
                pages = pages.toMutableList().apply { add(index = target, element = removeAt(index = position)) },
            )
        }
    }

    fun setTrashOriginal(trash: Boolean) {
        mutableState.update { it?.copy(trashOriginal = trash) }
    }

    fun cancel() {
        mutableState.value = null
    }

    /** Writes the edited pages and splits, asking first when the original goes to the trash. */
    fun apply() {
        val editor = mutableState.value ?: return
        val kept = editor.pages.filterNot { it.deleted }
        if (kept.isEmpty()) {
            return
        }
        if (editor.splits > 0 && editor.trashOriginal) {
            dialogs.ask(
                request = ConfirmationRequestModel(
                    title = "Split ${editor.name} into ${editor.splits + 1} documents",
                    message = "The new documents are created first; then ${editor.name} goes to the trash in " +
                        "every folder.",
                    affected = listOf(editor.name),
                    confirmLabel = "Split and move to trash",
                ),
            ) { run(editor = editor, kept = kept) }
        } else {
            run(editor = editor, kept = kept)
        }
    }

    private fun run(editor: PageEditorStateModel, kept: List<PageItemModel>) {
        val started = session.perform(label = "Editing the pages of ${editor.name}") {
            if (editor.edited) {
                val pages = kept.map { PageChangeModel(page = it.source, rotation = it.rotation) }
                val outcome = services.pdf.editPages(documentId = editor.documentId, pages = pages)
                if (outcome !is OperationOutcome.Done) {
                    session.report(message = OutcomeMessages.of(outcome = outcome, done = ""))
                    return@perform
                }
            }
            val ranges = rangesOf(kept = kept)
            if (ranges.size < 2) {
                session.report(message = UiMessageModel(text = "Saved the pages of ${editor.name}"))
                return@perform
            }
            val outcome = services.pdf.splitDocument(
                documentId = editor.documentId,
                ranges = ranges,
                trashOriginal = editor.trashOriginal,
            )
            session.report(
                message = OutcomeMessages.of(
                    outcome = outcome,
                    done = "Split ${editor.name} into ${ranges.size} documents",
                ),
            )
        }
        if (started) {
            mutableState.value = null
        }
    }

    /** The page ranges of the new version, cut after every page marked to split. */
    private fun rangesOf(kept: List<PageItemModel>): List<IntRange> {
        val ranges = mutableListOf<IntRange>()
        var start = 0
        kept.forEachIndexed { index, page ->
            if (page.splitAfter && index < kept.lastIndex) {
                ranges += start..index
                start = index + 1
            }
        }
        ranges += start..kept.lastIndex
        return ranges
    }

    private fun change(position: Int, change: (PageItemModel) -> PageItemModel) {
        mutableState.update { current ->
            current?.copy(
                pages = current.pages.mapIndexed { index, page -> if (index == position) change(page) else page },
            )
        }
    }

    private companion object {
        const val THUMBNAIL = 220
        const val FULL_TURN = 360
    }
}
