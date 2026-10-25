package dms.ui.library.drag

import androidx.compose.foundation.layout.offset
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.unit.round
import dms.domain.document.model.DocumentIdModel
import dms.ui.library.drag.model.DragItem
import dms.ui.library.model.CategoryEntryModel

/**
 * Shows the [DragPreview] next to the pointer while a document or category is dragged; [origin] is
 * the overlay's root position. A ticked document drags all [selected] documents along.
 */
@Composable
fun DragOverlay(
    drag: DocumentDrag,
    origin: Offset,
    categories: List<CategoryEntryModel>,
    selected: Set<DocumentIdModel> = emptySet(),
) {
    drag.dragged?.let { item ->
        val dragsSelection = item is DragItem.DraggedDocument && item.document.id in selected
        DragPreview(
            item = item,
            count = if (dragsSelection) selected.size else 1,
            action = drag.hovered?.let { DropTexts.action(item = item, target = it, categories = categories) },
            modifier = Modifier.offset { (drag.pointer - origin + Offset(x = 16f, y = 12f)).round() },
        )
    }
}
