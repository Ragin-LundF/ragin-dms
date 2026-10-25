package dms.ui.library.drag

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import dms.ui.library.drag.model.DragItem
import dms.ui.library.drag.model.DropTarget

/**
 * The document or category the user drags inside the library. Drop targets register their bounds in
 * window coordinates; the target under the pointer that takes the dragged item is [hovered], and
 * releasing over it calls [onDrop]. A category only goes into another category or to the top level.
 */
class DocumentDrag(private val onDrop: (DragItem, DropTarget) -> Unit) {

    private val targets = mutableMapOf<DropTarget, Rect>()

    var dragged: DragItem? by mutableStateOf(value = null)
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

    fun start(item: DragItem, at: Offset) {
        dragged = item
        pointer = at
        hovered = targetAt(position = at)
    }

    fun move(by: Offset) {
        pointer += by
        hovered = targetAt(position = pointer)
    }

    /** Ends the drag; drops the item if it is over a target that takes it. */
    fun finish() {
        val item = dragged
        val target = hovered
        cancel()
        if (item != null && target != null) {
            onDrop(item, target)
        }
    }

    fun cancel() {
        dragged = null
        hovered = null
    }

    private fun targetAt(position: Offset): DropTarget? {
        val item = dragged ?: return null
        return targets.entries.firstOrNull { (target, bounds) ->
            bounds.contains(offset = position) && takes(item = item, target = target)
        }?.key
    }

    private fun takes(item: DragItem, target: DropTarget): Boolean {
        return when (item) {
            is DragItem.DraggedDocument -> target != DropTarget.TopLevelCategory
            is DragItem.DraggedCategory -> target == DropTarget.TopLevelCategory ||
                (target is DropTarget.Category && target.categoryId != item.category.id)
            // Into one of its own subfolders is taken too, so the drop can say why it is refused.
            is DragItem.DraggedFolder -> target is DropTarget.Folder && target.path != item.path
        }
    }
}
