package dms.ui.library.folders.view

import androidx.compose.foundation.border
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.input.TextFieldValue
import dms.ui.theme.Spacing

/**
 * A folder name typed in place, focused and selected when it appears. Every change goes to [onType];
 * Enter or losing the focus saves it ([onSave]) and Esc cancels ([onCancel]), whichever comes first.
 */
@Composable
fun FolderNameField(
    initial: String,
    onType: (String) -> Unit,
    onSave: (String) -> Unit,
    onCancel: () -> Unit,
    modifier: Modifier = Modifier,
) {
    var value by remember {
        mutableStateOf(value = TextFieldValue(text = initial, selection = TextRange(start = 0, end = initial.length)))
    }
    var focused by remember { mutableStateOf(value = false) }
    var done by remember { mutableStateOf(value = false) }
    val finish = { action: () -> Unit ->
        if (!done) {
            done = true
            action()
        }
    }
    val focus = remember { FocusRequester() }
    LaunchedEffect(Unit) { focus.requestFocus() }
    val colors = MaterialTheme.colorScheme
    BasicTextField(
        value = value,
        onValueChange = {
            value = it
            onType(it.text)
        },
        singleLine = true,
        textStyle = MaterialTheme.typography.bodyMedium.copy(color = colors.onSurface),
        cursorBrush = SolidColor(value = colors.primary),
        modifier = modifier
            .fillMaxWidth()
            .focusRequester(focusRequester = focus)
            .onFocusChanged { state ->
                if (focused && !state.isFocused) {
                    finish { onSave(value.text) }
                }
                focused = state.isFocused
            }
            .onPreviewKeyEvent { event ->
                if (event.type != KeyEventType.KeyDown) {
                    return@onPreviewKeyEvent false
                }
                when (event.key) {
                    Key.Enter, Key.NumPadEnter -> {
                        finish { onSave(value.text) }
                        true
                    }

                    Key.Escape -> {
                        finish(onCancel)
                        true
                    }

                    else -> false
                }
            }
            .border(width = Spacing.hairline, color = colors.primary, shape = MaterialTheme.shapes.extraSmall)
            .padding(horizontal = Spacing.small, vertical = Spacing.xsmall),
    )
}
