package dms.ui.components.layout

import androidx.compose.foundation.gestures.Orientation
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.gestures.draggable
import androidx.compose.foundation.gestures.rememberDraggableState
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.width
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.pointerHoverIcon
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.Dp
import dms.ui.theme.Spacing

/**
 * A divider between two panes that resizes the pane to its left when dragged sideways. [onResizeEnd]
 * runs once when the drag ends, so a caller can store the width then instead of on every step;
 * [onDoubleClick] lets a double-click reset the pane.
 */
@Composable
fun ResizeHandle(
    onResize: (Dp) -> Unit,
    modifier: Modifier = Modifier,
    onResizeEnd: () -> Unit = {},
    onDoubleClick: (() -> Unit)? = null,
) {
    val density = LocalDensity.current
    val currentOnResize by rememberUpdatedState(newValue = onResize)
    val currentOnResizeEnd by rememberUpdatedState(newValue = onResizeEnd)
    val currentOnDoubleClick by rememberUpdatedState(newValue = onDoubleClick)
    val state = rememberDraggableState { delta -> currentOnResize(with(density) { delta.toDp() }) }
    Box(
        modifier = modifier
            .fillMaxHeight()
            .width(width = Spacing.small)
            .pointerHoverIcon(icon = LocalResizePointerIcon.current)
            .pointerInput(Unit) {
                detectTapGestures(onDoubleTap = { currentOnDoubleClick?.invoke() })
            }
            .draggable(
                state = state,
                orientation = Orientation.Horizontal,
                onDragStopped = { currentOnResizeEnd() },
            ),
        contentAlignment = Alignment.Center,
    ) {
        VerticalDivider()
    }
}
