package dms.ui.library.drag

import androidx.compose.foundation.layout.offset
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.unit.round
import dms.ui.library.model.CategoryEntry

/**
 * Shows the [DragPreview] next to the pointer while a document or category is dragged; [origin] is
 * the overlay's root position.
 */
@Composable
fun DragOverlay(drag: DocumentDrag, origin: Offset, categories: List<CategoryEntry>) {
    drag.dragged?.let { item ->
        DragPreview(
            item = item,
            action = drag.hovered?.let { DropTexts.action(item = item, target = it, categories = categories) },
            modifier = Modifier.offset { (drag.pointer - origin + Offset(x = 16f, y = 12f)).round() },
        )
    }
}
