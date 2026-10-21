package dms.ui.library

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.ArrowDropDown
import androidx.compose.material.icons.rounded.CalendarMonth
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import dms.ui.library.model.YearFilter
import dms.ui.theme.Spacing

/** Filters the documents by year: of the document date, or of when a document without one was added. */
@Composable
fun YearMenu(years: List<Int>, selected: YearFilter?, onSelect: (YearFilter?) -> Unit) {
    var open by remember { mutableStateOf(value = false) }
    Box {
        FilterChip(
            selected = selected != null,
            onClick = { open = true },
            label = { Text(text = selected?.label ?: "All years") },
            leadingIcon = {
                Icon(
                    imageVector = Icons.Rounded.CalendarMonth,
                    contentDescription = null,
                    modifier = Modifier.size(size = Spacing.icon),
                )
            },
            trailingIcon = { Icon(imageVector = Icons.Rounded.ArrowDropDown, contentDescription = null) },
        )
        DropdownMenu(expanded = open, onDismissRequest = { open = false }) {
            DropdownMenuItem(
                text = { Text(text = "All years") },
                onClick = {
                    open = false
                    onSelect(null)
                },
            )
            years.forEach { year ->
                DropdownMenuItem(
                    text = { Text(text = year.toString()) },
                    onClick = {
                        open = false
                        onSelect(YearFilter(year = year))
                    },
                )
            }
        }
    }
}
