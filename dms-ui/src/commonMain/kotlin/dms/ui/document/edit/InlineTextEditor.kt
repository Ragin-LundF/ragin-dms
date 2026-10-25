package dms.ui.document.edit

import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SuggestionChip
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.TextFieldValue
import dms.ui.theme.Spacing

/**
 * A value edited in place: it starts focused with [initial] selected, Enter saves, Esc cancels, and
 * an unchanged value cancels too. [suggestionsFor] offers values for what is typed; a click on one
 * saves it. Losing the focus keeps the edit open, so a refused value can still be fixed.
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun InlineTextEditor(
    initial: String,
    description: String,
    onSave: (String) -> Unit,
    onCancel: () -> Unit,
    suggestionsFor: (String) -> List<String> = { emptyList() },
) {
    val colors = MaterialTheme.colorScheme
    var field by remember(key1 = initial) {
        mutableStateOf(value = TextFieldValue(text = initial, selection = TextRange(start = 0, end = initial.length)))
    }
    val focus = remember { FocusRequester() }
    LaunchedEffect(key1 = Unit) { focus.requestFocus() }
    val commit = {
        if (field.text.trim() == initial.trim()) onCancel() else onSave(field.text)
    }
    Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.xsmall)) {
        BasicTextField(
            value = field,
            onValueChange = { field = it },
            singleLine = true,
            textStyle = MaterialTheme.typography.bodySmall.copy(color = colors.onSurface),
            cursorBrush = SolidColor(value = colors.primary),
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
            keyboardActions = KeyboardActions(onDone = { commit() }),
            modifier = Modifier
                .fillMaxWidth()
                .focusRequester(focusRequester = focus)
                .onPreviewKeyEvent { event ->
                    if (event.type != KeyEventType.KeyDown) {
                        return@onPreviewKeyEvent false
                    }
                    when (event.key) {
                        Key.Escape -> onCancel()
                        Key.Enter, Key.NumPadEnter -> commit()
                        else -> return@onPreviewKeyEvent false
                    }
                    true
                }
                .semantics { contentDescription = description }
                .border(width = Spacing.hairline, color = colors.primary, shape = MaterialTheme.shapes.extraSmall)
                .padding(horizontal = Spacing.xsmall, vertical = Spacing.hairline * 2),
        )
        val suggestions = suggestionsFor(field.text)
        if (suggestions.isNotEmpty()) {
            FlowRow(horizontalArrangement = Arrangement.spacedBy(space = Spacing.xsmall)) {
                suggestions.forEach { suggestion ->
                    SuggestionChip(
                        onClick = { onSave(suggestion) },
                        label = { Text(text = suggestion, maxLines = 1, softWrap = false) },
                    )
                }
            }
        }
    }
}
