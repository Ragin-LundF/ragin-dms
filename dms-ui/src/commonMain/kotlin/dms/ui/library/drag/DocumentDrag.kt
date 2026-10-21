package dms.ui.library.drag

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import dms.domain.document.Document
import dms.ui.library.drag.model.DropTarget

/**
 * The document the user drags inside the library. Drop targets register their bounds in window
 * coordinates; the target under the pointer is [hovered], and releasing over it calls [onDrop].
 */
class DocumentDrag(private val onDrop: (Document, DropTarget) -> Unit) {

    private val targets = mutableMapOf<DropTarget, Rect>()

    var dragged: Document? by mutableStateOf(value = null)
        private set

    var pointer: Offset by mutableStateOf(value = Offset.Zero)
        private set

    var hovered: DropTarget? by mutableStateOf(value = null)
        private set

    fun register(target: DropTarget, bounds: Rect) {
        targets[target] = bounds
    }

    fun unregister(target: DropTarget) {
        targets.remove(key = target)
    }

    fun start(document: Document, at: Offset) {
        dragged = document
        pointer = at
        hovered = targetAt(position = at)
    }

    fun move(by: Offset) {
        pointer += by
        hovered = targetAt(position = pointer)
    }

    /** Ends the drag; drops the document if it is over a target. */
    fun finish() {
        val document = dragged
        val target = hovered
        cancel()
        if (document != null && target != null) {
            onDrop(document, target)
        }
    }

    fun cancel() {
        dragged = null
        hovered = null
    }

    private fun targetAt(position: Offset): DropTarget? {
        return targets.entries.firstOrNull { (_, bounds) -> bounds.contains(offset = position) }?.key
    }
}
