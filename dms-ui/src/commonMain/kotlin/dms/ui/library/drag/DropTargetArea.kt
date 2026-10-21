package dms.ui.library.drag

import androidx.compose.foundation.layout.Box
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.boundsInRoot
import androidx.compose.ui.layout.onGloballyPositioned
import dms.ui.library.drag.model.DropTarget

/** Registers [content] as the drop [target]; [content] learns whether a document hovers over it. */
@Composable
fun DropTargetArea(drag: DocumentDrag, target: DropTarget, content: @Composable (hovered: Boolean) -> Unit) {
    DisposableEffect(drag, target) {
        onDispose { drag.unregister(target = target) }
    }
    Box(modifier = Modifier.onGloballyPositioned { drag.register(target = target, bounds = it.boundsInRoot()) }) {
        content(drag.dragged != null && drag.hovered == target)
    }
}
