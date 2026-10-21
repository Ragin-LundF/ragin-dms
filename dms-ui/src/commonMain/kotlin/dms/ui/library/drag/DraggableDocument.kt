package dms.ui.library.drag

import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInRoot
import dms.domain.document.Document

/** Makes [content] draggable as [document]; a click still selects it, a drag starts after a few pixels. */
@Composable
fun DraggableDocument(
    drag: DocumentDrag,
    document: Document,
    modifier: Modifier = Modifier,
    content: @Composable () -> Unit,
) {
    val origin = remember { mutableStateOf(value = Offset.Zero) }
    val current by rememberUpdatedState(newValue = document)
    Box(
        modifier = modifier
            .onGloballyPositioned { origin.value = it.positionInRoot() }
            .pointerInput(drag) {
                detectDragGestures(
                    onDragStart = { offset -> drag.start(document = current, at = origin.value + offset) },
                    onDragEnd = { drag.finish() },
                    onDragCancel = { drag.cancel() },
                    onDrag = { change, amount ->
                        change.consume()
                        drag.move(by = amount)
                    },
                )
            },
    ) {
        content()
    }
}
