package dms.ui.library.documents

import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.waitForUpOrCancellation
import androidx.compose.foundation.layout.Box
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.PointerKeyboardModifiers
import androidx.compose.ui.input.pointer.isCtrlPressed
import androidx.compose.ui.input.pointer.isMetaPressed
import androidx.compose.ui.input.pointer.isShiftPressed
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.onClick
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import dms.ui.library.documents.model.DocumentClick

/**
 * Makes [content] clickable as one document and tells [onClick] which keys were held: ⌘ or Ctrl ticks
 * it, Shift ticks a range. A drag that starts on it is no click.
 */
@Composable
fun ClickableDocument(
    label: String,
    onClick: (DocumentClick) -> Unit,
    modifier: Modifier = Modifier,
    content: @Composable () -> Unit,
) {
    val current by rememberUpdatedState(newValue = onClick)
    Box(
        modifier = modifier
            .pointerInput(Unit) {
                awaitEachGesture {
                    awaitFirstDown()
                    val up = waitForUpOrCancellation() ?: return@awaitEachGesture
                    up.consume()
                    current(clickOf(keys = currentEvent.keyboardModifiers))
                }
            }
            .semantics {
                role = Role.Button
                onClick(label = label) {
                    current(DocumentClick.PLAIN)
                    true
                }
            },
    ) {
        content()
    }
}

private fun clickOf(keys: PointerKeyboardModifiers): DocumentClick {
    return when {
        keys.isShiftPressed -> DocumentClick.RANGE
        keys.isMetaPressed || keys.isCtrlPressed -> DocumentClick.TOGGLE
        else -> DocumentClick.PLAIN
    }
}
