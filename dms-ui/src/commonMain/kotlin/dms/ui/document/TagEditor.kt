package dms.ui.document

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.InputChip
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.SuggestionChip
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.ImeAction
import dms.domain.taxonomy.TagName
import dms.ui.theme.Spacing
import java.util.Locale

/** The tags of a document as chips; typing suggests tags the library already uses. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun TagEditor(
    tags: List<TagName>,
    allTags: List<TagName>,
    enabled: Boolean,
    onAdd: (String) -> Unit,
    onRemove: (TagName) -> Unit,
) {
    var text by remember { mutableStateOf(value = "") }
    val prefix = text.trim().lowercase(Locale.ROOT)
    val suggestions = allTags
        .filter { it !in tags && prefix.isNotEmpty() && it.value.lowercase(Locale.ROOT).startsWith(prefix = prefix) }
        .take(n = 6)
    val add = { value: String ->
        onAdd(value)
        text = ""
    }
    Column {
        FlowRow(horizontalArrangement = Arrangement.spacedBy(space = Spacing.xsmall)) {
            tags.forEach { tag ->
                InputChip(
                    selected = false,
                    onClick = { onRemove(tag) },
                    enabled = enabled,
                    label = { Text(text = "${tag.value}  ×", maxLines = 1, softWrap = false) },
                )
            }
        }
        OutlinedTextField(
            value = text,
            onValueChange = { text = it },
            label = { Text(text = "Add tag") },
            singleLine = true,
            enabled = enabled,
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
            keyboardActions = KeyboardActions(onDone = { add(text) }),
            modifier = Modifier.fillMaxWidth(),
        )
        FlowRow(horizontalArrangement = Arrangement.spacedBy(space = Spacing.xsmall)) {
            suggestions.forEach { tag ->
                SuggestionChip(onClick = { add(tag.value) }, label = { Text(text = tag.value, softWrap = false) })
            }
        }
    }
}
