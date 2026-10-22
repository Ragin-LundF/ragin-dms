package dms.ui.library.drag

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.domain.document.model.DocumentModel
import dms.ui.library.drag.model.DragItem

/** Makes [content] draggable as [document]; a click still selects it, a drag starts after a few pixels. */
@Composable
fun DraggableDocument(
    drag: DocumentDrag,
    document: DocumentModel,
    modifier: Modifier = Modifier,
    content: @Composable () -> Unit,
) {
    DraggableItem(
        drag = drag,
        item = DragItem.DraggedDocument(document = document),
        modifier = modifier,
        content = content,
    )
}
