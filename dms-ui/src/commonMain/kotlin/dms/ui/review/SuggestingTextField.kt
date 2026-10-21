package dms.ui.review

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.ArrowDropDown
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.PopupProperties

/**
 * A text field that offers [options] in a menu: typing shows the options containing the text, the arrow
 * shows all of them. Any other text may still be typed.
 */
@Composable
fun SuggestingTextField(value: String, label: String, options: List<String>, onValueChange: (String) -> Unit) {
    var open by remember { mutableStateOf(value = false) }
    var showAll by remember { mutableStateOf(value = false) }
    val shown = if (showAll) options else options.filter { it.contains(other = value.trim(), ignoreCase = true) }
    Box {
        OutlinedTextField(
            value = value,
            onValueChange = {
                onValueChange(it)
                open = true
                showAll = false
            },
            label = { Text(text = label) },
            singleLine = true,
            trailingIcon = {
                IconButton(
                    onClick = {
                        showAll = !open || !showAll
                        open = showAll
                    },
                ) {
                    Icon(imageVector = Icons.Rounded.ArrowDropDown, contentDescription = "Show all")
                }
            },
            modifier = Modifier.fillMaxWidth(),
        )
        DropdownMenu(
            expanded = open && shown.isNotEmpty(),
            onDismissRequest = { open = false },
            properties = PopupProperties(focusable = false),
            modifier = Modifier.heightIn(max = 320.dp),
        ) {
            shown.forEach { option ->
                DropdownMenuItem(
                    text = { Text(text = option, maxLines = 1, softWrap = false, overflow = TextOverflow.Ellipsis) },
                    onClick = {
                        onValueChange(option)
                        open = false
                    },
                )
            }
        }
    }
}
