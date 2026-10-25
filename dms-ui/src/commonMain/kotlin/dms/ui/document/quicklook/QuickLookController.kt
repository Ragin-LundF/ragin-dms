package dms.ui.document.quicklook

import dms.domain.document.model.DocumentIdModel
import dms.ui.document.model.QuickFact
import dms.ui.document.model.QuickLookStateModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update

/**
 * Which document the quick look shows, and the fact edited in place. The caller hands in the
 * documents in view order; next and previous move through them and stop at the ends. Showing
 * another document ends an edit without saving it.
 */
class QuickLookController {

    private val mutableState = MutableStateFlow(value = QuickLookStateModel())

    val state: StateFlow<QuickLookStateModel> = mutableState.asStateFlow()

    val isOpen: Boolean
        get() {
            return mutableState.value.shown != null
        }

    fun open(documentId: DocumentIdModel, order: List<DocumentIdModel>) {
        mutableState.value = QuickLookStateModel(shown = documentId, order = order)
    }

    /** Space: closes an open quick look, or opens it on [documentId]; without a document nothing opens. */
    fun toggle(documentId: DocumentIdModel?, order: List<DocumentIdModel>) {
        when {
            isOpen -> close()
            documentId != null -> open(documentId = documentId, order = order)
        }
    }

    /** Esc: closes the quick look; `false` when it was closed already, so the key can do its next job. */
    fun close(): Boolean {
        val wasOpen = isOpen
        mutableState.value = QuickLookStateModel()
        return wasOpen
    }

    /** Takes the new view order, for example after a filter changed; the shown document stays. */
    fun updateOrder(order: List<DocumentIdModel>) {
        mutableState.update { it.copy(order = order) }
    }

    /** ↓: the next document in view order; nothing at the end or when the shown one left the view. */
    fun next() {
        step(by = 1)
    }

    /** ↑: the previous document in view order. */
    fun previous() {
        step(by = -1)
    }

    fun edit(fact: QuickFact) {
        mutableState.update { it.copy(editing = fact) }
    }

    fun endEdit() {
        mutableState.update { it.copy(editing = null) }
    }

    private fun step(by: Int) {
        val current = mutableState.value
        val shown = current.shown ?: return
        val index = current.order.indexOf(element = shown)
        if (index < 0) {
            return
        }
        val target = current.order.getOrNull(index = index + by) ?: return
        mutableState.value = current.copy(shown = target, editing = null)
    }
}
