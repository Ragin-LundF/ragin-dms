package dms.ui.library.filter

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEvent
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import dms.ui.components.ReasonTooltip
import dms.ui.library.filter.model.FacetFilter
import dms.ui.library.filter.model.FilterTokenModel
import dms.ui.theme.LibrarySizes
import dms.ui.theme.Spacing

/**
 * The search field. The active filters stand in it as tokens with an ×, a saved view's own conditions
 * as tokens without one. Typing suggests tokens ("+ tag: invoice"), Enter takes the first, and
 * Backspace in the empty field removes the last removable token. The field takes the focus whenever
 * [focusRequests] grows, as "/" asks.
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun TokenSearchField(
    query: String,
    tokens: List<FilterTokenModel>,
    suggestions: List<FilterTokenModel>,
    placeholder: String,
    focusRequests: Int,
    onQuery: (String) -> Unit,
    onRemove: (FacetFilter) -> Unit,
    onTake: (FacetFilter) -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = MaterialTheme.colorScheme
    val focus = remember { FocusRequester() }
    var focused by remember { mutableStateOf(value = false) }
    LaunchedEffect(key1 = focusRequests) {
        if (focusRequests > 0) {
            focus.requestFocus()
        }
    }
    val keys: (KeyEvent) -> Boolean = { event ->
        handleKey(
            event = event,
            query = query,
            tokens = tokens,
            suggestions = suggestions,
            onRemove = onRemove,
            onTake = onTake,
        )
    }
    Surface(
        modifier = modifier.fillMaxWidth(),
        shape = MaterialTheme.shapes.medium,
        color = colors.surface,
        border = BorderStroke(width = Spacing.hairline, color = if (focused) colors.primary else colors.outlineVariant),
    ) {
        FlowRow(
            modifier = Modifier.padding(horizontal = Spacing.medium, vertical = Spacing.xsmall + Spacing.hairline),
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
            verticalArrangement = Arrangement.spacedBy(space = Spacing.xsmall),
            itemVerticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                imageVector = Icons.Rounded.Search,
                contentDescription = null,
                tint = colors.onSurfaceVariant,
                modifier = Modifier.size(size = Spacing.icon),
            )
            tokens.forEach { token -> FilterToken(token = token, onRemove = onRemove) }
            QueryInput(
                query = query,
                placeholder = placeholder,
                onQuery = onQuery,
                modifier = Modifier
                    .weight(weight = 1f)
                    .widthIn(min = LibrarySizes.searchInputMinWidth)
                    .focusRequester(focusRequester = focus)
                    .onFocusChanged { focused = it.isFocused }
                    .onPreviewKeyEvent(onPreviewKeyEvent = keys),
            )
            suggestions.forEach { suggestion ->
                TextButton(onClick = { suggestion.filter?.let(onTake) }) { Text(text = "+ ${suggestion.label}") }
            }
            KeyHint(text = "/")
        }
    }
}

@Composable
private fun QueryInput(query: String, placeholder: String, onQuery: (String) -> Unit, modifier: Modifier) {
    val colors = MaterialTheme.colorScheme
    BasicTextField(
        value = query,
        onValueChange = onQuery,
        singleLine = true,
        textStyle = MaterialTheme.typography.bodyMedium.copy(color = colors.onSurface),
        cursorBrush = SolidColor(value = colors.primary),
        modifier = modifier
            .padding(vertical = Spacing.small)
            .semantics { contentDescription = "Search the documents" },
        decorationBox = { field ->
            Box {
                if (query.isEmpty()) {
                    Text(
                        text = placeholder,
                        style = MaterialTheme.typography.bodyMedium,
                        color = colors.onSurfaceVariant,
                        maxLines = 1,
                    )
                }
                field()
            }
        },
    )
}

/** Enter takes the first suggestion; Backspace in the empty field removes the last removable token. */
private fun handleKey(
    event: KeyEvent,
    query: String,
    tokens: List<FilterTokenModel>,
    suggestions: List<FilterTokenModel>,
    onRemove: (FacetFilter) -> Unit,
    onTake: (FacetFilter) -> Unit,
): Boolean {
    if (event.type != KeyEventType.KeyDown) {
        return false
    }
    val first = suggestions.firstNotNullOfOrNull { it.filter }
    if (event.key == Key.Enter && first != null) {
        onTake(first)
        return true
    }
    val last = tokens.lastOrNull { it.filter != null }?.filter
    if (event.key == Key.Backspace && query.isEmpty() && last != null) {
        onRemove(last)
        return true
    }
    return false
}

@Composable
private fun FilterToken(token: FilterTokenModel, onRemove: (FacetFilter) -> Unit) {
    val colors = MaterialTheme.colorScheme
    val removable = token.filter
    ReasonTooltip(reason = "Part of the saved view".takeIf { removable == null }) {
        Surface(
            shape = MaterialTheme.shapes.small,
            color = colors.primaryContainer,
            contentColor = colors.onPrimaryContainer,
        ) {
            Row(
                modifier = Modifier.padding(
                    start = Spacing.small,
                    end = if (removable == null) Spacing.small else Spacing.xsmall,
                ),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(space = Spacing.xsmall),
            ) {
                Text(
                    text = "${token.key}:",
                    style = MaterialTheme.typography.labelMedium,
                    color = colors.onPrimaryContainer.copy(alpha = 0.7f),
                    maxLines = 1,
                    softWrap = false,
                )
                Text(
                    text = token.value,
                    style = MaterialTheme.typography.labelMedium,
                    maxLines = 1,
                    softWrap = false,
                    modifier = Modifier.padding(vertical = Spacing.xsmall),
                )
                removable?.let { filter ->
                    Icon(
                        imageVector = Icons.Rounded.Close,
                        contentDescription = "Remove ${token.label}",
                        modifier = Modifier
                            .clip(shape = MaterialTheme.shapes.extraSmall)
                            .clickable { onRemove(filter) }
                            .padding(all = Spacing.hairline * 2)
                            .size(size = LibrarySizes.labelIcon),
                    )
                }
            }
        }
    }
}

@Composable
private fun KeyHint(text: String) {
    val colors = MaterialTheme.colorScheme
    Surface(
        shape = MaterialTheme.shapes.extraSmall,
        color = colors.surface,
        border = BorderStroke(width = Spacing.hairline, color = colors.outlineVariant),
    ) {
        Text(
            text = text,
            style = MaterialTheme.typography.labelSmall,
            color = colors.onSurfaceVariant,
            modifier = Modifier.padding(horizontal = Spacing.xsmall + Spacing.hairline),
        )
    }
}
