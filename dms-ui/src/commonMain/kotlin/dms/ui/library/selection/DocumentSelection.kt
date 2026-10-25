package dms.ui.library.selection

import dms.domain.document.model.DocumentIdModel
import dms.ui.library.selection.model.DocumentSelectionModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update

/**
 * The documents ticked for a bulk action. A tick or a ⌘/Ctrl-click toggles one document, a Shift-click
 * adds every document from the last one ticked to the clicked one, in the order [shown] lists them.
 */
class DocumentSelection(private val shown: () -> List<DocumentIdModel>) {

    private val mutableState = MutableStateFlow(value = DocumentSelectionModel())

    val state: StateFlow<DocumentSelectionModel> = mutableState.asStateFlow()

    fun toggle(documentId: DocumentIdModel) {
        mutableState.update { state ->
            val ticked = documentId in state.selected
            DocumentSelectionModel(
                selected = if (ticked) state.selected - documentId else state.selected + documentId,
                anchor = documentId,
            )
        }
    }

    /** Adds the documents from the anchor to [documentId]; without an anchor, only [documentId]. */
    fun extend(documentId: DocumentIdModel) {
        val order = shown()
        mutableState.update { state ->
            val from = state.anchor?.let { order.indexOf(element = it) }?.takeIf { it >= 0 }
            val to = order.indexOf(element = documentId)
            if (from == null || to < 0) {
                return@update DocumentSelectionModel(selected = state.selected + documentId, anchor = documentId)
            }
            val range = order.subList(fromIndex = minOf(from, to), toIndex = maxOf(from, to) + 1)
            state.copy(selected = state.selected + range)
        }
    }

    fun selectAll() {
        mutableState.value = DocumentSelectionModel(selected = shown().toSet())
    }

    fun clear() {
        mutableState.value = DocumentSelectionModel()
    }
}
