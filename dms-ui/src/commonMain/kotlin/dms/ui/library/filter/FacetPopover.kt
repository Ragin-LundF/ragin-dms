package dms.ui.library.filter

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.library.filter.model.Facet
import dms.ui.library.filter.model.FacetFilter
import dms.ui.library.filter.model.FacetOptionModel
import dms.ui.theme.Spacing

/**
 * The content of a facet's popover: a field that narrows the values, the values with their counts
 * (subcategories and subfolders indented, the [active] one checked), and a note on how the list works.
 * [optionMenu] adds a ⋯ menu to a value, [footer] an action below the list.
 */
@Composable
fun ColumnScope.FacetPopover(
    facet: Facet,
    options: List<FacetOptionModel>,
    active: FacetFilter?,
    optionMenu: @Composable (FacetOptionModel) -> Unit,
    footer: @Composable () -> Unit,
    onPick: (FacetFilter) -> Unit,
) {
    var narrowed by remember { mutableStateOf(value = "") }
    NarrowField(text = narrowed, placeholder = "Filter ${facet.label.lowercase()}…") { narrowed = it }
    val shown = options.filter { it.label.contains(other = narrowed.trim(), ignoreCase = true) }
    if (shown.isEmpty()) {
        Text(
            text = "Nothing to choose",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.padding(horizontal = Spacing.medium, vertical = Spacing.small),
        )
    }
    shown.forEach { option ->
        OptionItem(option = option, checked = option.filter == active, menu = { optionMenu(option) }) {
            onPick(option.filter)
        }
    }
    footer()
    noteOf(facet = facet)?.let { note ->
        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
        Text(
            text = note,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.padding(horizontal = Spacing.medium, vertical = Spacing.small),
        )
    }
}

@Composable
private fun OptionItem(option: FacetOptionModel, checked: Boolean, menu: @Composable () -> Unit, onClick: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    DropdownMenuItem(
        text = {
            Text(
                text = option.label,
                fontStyle = if (option.missing) FontStyle.Italic else FontStyle.Normal,
                color = if (checked) colors.primary else colors.onSurface,
                maxLines = 1,
                softWrap = false,
                overflow = TextOverflow.Ellipsis,
            )
        },
        leadingIcon = if (checked) {
            { Icon(imageVector = Icons.Rounded.Check, contentDescription = "Active", tint = colors.primary) }
        } else {
            null
        },
        trailingIcon = {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(space = Spacing.xsmall),
            ) {
                Text(
                    text = option.count.toString(),
                    style = MaterialTheme.typography.labelSmall,
                    color = colors.onSurfaceVariant,
                )
                menu()
            }
        },
        contentPadding = PaddingValues(start = Spacing.medium + Spacing.large * option.depth, end = Spacing.small),
        onClick = onClick,
    )
}

@Composable
private fun NarrowField(text: String, placeholder: String, onChange: (String) -> Unit) {
    val colors = MaterialTheme.colorScheme
    Surface(
        shape = MaterialTheme.shapes.small,
        color = colors.surfaceContainer,
        modifier = Modifier.fillMaxWidth().padding(horizontal = Spacing.small, vertical = Spacing.xsmall),
    ) {
        BasicTextField(
            value = text,
            onValueChange = onChange,
            singleLine = true,
            textStyle = MaterialTheme.typography.bodyMedium.copy(color = colors.onSurface),
            cursorBrush = SolidColor(value = colors.primary),
            modifier = Modifier.padding(all = Spacing.small).semantics { contentDescription = placeholder },
            decorationBox = { field ->
                Box {
                    if (text.isEmpty()) {
                        Text(
                            text = placeholder,
                            style = MaterialTheme.typography.bodyMedium,
                            color = colors.onSurfaceVariant,
                        )
                    }
                    field()
                }
            },
        )
    }
}

private fun noteOf(facet: Facet): String? {
    return when (facet) {
        Facet.CATEGORY -> "Subcategories stay inside the list; a category includes them."
        Facet.TAG -> "Sorted by how often a tag is used."
        Facet.FOLDER -> "Shows what is inside a folder. To move, rename or create folders, open Folders " +
            "in the sidebar."
        Facet.YEAR -> "The year of the document date, or of when a document without one was added."
        Facet.CORRESPONDENT,
        Facet.TYPE,
        -> null
    }
}
