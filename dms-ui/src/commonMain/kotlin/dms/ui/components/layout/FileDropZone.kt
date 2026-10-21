package dms.ui.components.layout

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.draganddrop.dragAndDropTarget
import androidx.compose.foundation.layout.Box
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draganddrop.DragAndDropEvent
import androidx.compose.ui.draganddrop.DragAndDropTarget
import dms.ui.PlatformActions

/**
 * Accepts files dragged in from the file manager. While files hover over it, it shows [label] in
 * an overlay; [onDrop] gets their absolute paths (empty if only folders were dropped).
 */
@OptIn(ExperimentalFoundationApi::class)
@Composable
fun FileDropZone(
    platform: PlatformActions,
    label: String,
    onDrop: (List<String>) -> Unit,
    modifier: Modifier = Modifier,
    content: @Composable () -> Unit,
) {
    var active by remember { mutableStateOf(value = false) }
    val currentOnDrop by rememberUpdatedState(newValue = onDrop)
    val target = remember(platform) {
        object : DragAndDropTarget {
            override fun onEntered(event: DragAndDropEvent) {
                active = true
            }

            override fun onExited(event: DragAndDropEvent) {
                active = false
            }

            override fun onEnded(event: DragAndDropEvent) {
                active = false
            }

            override fun onDrop(event: DragAndDropEvent): Boolean {
                active = false
                currentOnDrop(platform.droppedFiles(event = event))
                return true
            }
        }
    }
    Box(
        modifier = modifier.dragAndDropTarget(
            shouldStartDragAndDrop = { event -> platform.carriesFiles(event = event) },
            target = target,
        ),
    ) {
        content()
        if (active) {
            DropOverlay(label = label)
        }
    }
}
