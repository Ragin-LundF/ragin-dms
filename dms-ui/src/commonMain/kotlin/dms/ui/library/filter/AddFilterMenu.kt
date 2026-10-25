package dms.ui.library.filter

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material.icons.rounded.CheckBox
import androidx.compose.material.icons.rounded.CheckBoxOutlineBlank
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.drawscope.Stroke
import dms.ui.library.model.YearFilterModel
import dms.ui.theme.Spacing

/**
 * The dashed "+ Filter" chip and its menu: searching the documents' text on or off, and the year,
 * of the document date or of when a document without one was added.
 */
@Composable
fun AddFilterMenu(
    years: List<Int>,
    selected: YearFilterModel?,
    contentSearch: Boolean,
    onYear: (YearFilterModel?) -> Unit,
    onContentSearch: (Boolean) -> Unit,
) {
    var open by remember { mutableStateOf(value = false) }
    val checkbox = if (contentSearch) Icons.Rounded.CheckBox else Icons.Rounded.CheckBoxOutlineBlank
    val choose: (() -> Unit) -> Unit = { action ->
        open = false
        action()
    }
    Box {
        DashedChip(text = "+ Filter", onClick = { open = true })
        DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
            DropdownMenuItem(
                text = { Text(text = "Search in text") },
                leadingIcon = { Icon(imageVector = checkbox, contentDescription = null) },
                onClick = { choose { onContentSearch(!contentSearch) } },
            )
            HorizontalDivider()
            YearItem(text = "Every year", checked = selected == null) { choose { onYear(null) } }
            years.forEach { year ->
                YearItem(text = "Year $year", checked = selected == YearFilterModel(year = year)) {
                    choose { onYear(YearFilterModel(year = year)) }
                }
            }
        }
    }
}

@Composable
private fun YearItem(text: String, checked: Boolean, onClick: () -> Unit) {
    DropdownMenuItem(
        text = { Text(text = text) },
        trailingIcon = if (checked) {
            { Icon(imageVector = Icons.Rounded.Check, contentDescription = "Selected") }
        } else {
            null
        },
        onClick = onClick,
    )
}

@Composable
private fun DashedChip(text: String, onClick: () -> Unit) {
    val outline = MaterialTheme.colorScheme.outline
    Text(
        text = text,
        style = MaterialTheme.typography.labelLarge,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        maxLines = 1,
        softWrap = false,
        modifier = Modifier
            .clip(shape = CircleShape)
            .clickable(onClick = onClick)
            .drawBehind {
                val width = Spacing.hairline.toPx()
                val dash = Spacing.xsmall.toPx()
                drawRoundRect(
                    color = outline,
                    topLeft = Offset(x = width / 2, y = width / 2),
                    size = Size(width = size.width - width, height = size.height - width),
                    cornerRadius = CornerRadius(x = size.height / 2),
                    style = Stroke(
                        width = width,
                        pathEffect = PathEffect.dashPathEffect(intervals = floatArrayOf(dash, dash)),
                    ),
                )
            }
            .padding(horizontal = Spacing.medium, vertical = Spacing.xsmall + Spacing.hairline * 2),
    )
}
