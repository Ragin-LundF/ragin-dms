package dms.ui.document.details

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.unit.dp
import dms.domain.document.model.DocumentModel

/** A free-text note on the document, saved when the field loses the focus; search finds it too. */
@Composable
fun NoteEditor(document: DocumentModel, enabled: Boolean, onSave: (String) -> Unit) {
    var text by remember(key1 = document.id, key2 = document.note) { mutableStateOf(value = document.note.orEmpty()) }
    var focused by remember(key1 = document.id) { mutableStateOf(value = false) }
    OutlinedTextField(
        value = text,
        onValueChange = { text = it },
        label = { Text(text = "Note") },
        placeholder = { Text(text = "Anything worth remembering about this document") },
        enabled = enabled,
        minLines = 3,
        modifier = Modifier
            .fillMaxWidth()
            .heightIn(min = 96.dp)
            .onFocusChanged { state ->
                if (focused && !state.isFocused) {
                    onSave(text)
                }
                focused = state.isFocused
            },
    )
}
