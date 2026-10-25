package dms.ui.document.edit

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import dms.domain.document.model.DocumentModel
import dms.ui.document.model.MetadataDraftModel
import dms.ui.theme.Spacing

/** Title, document date and correspondent, saved together. */
@Composable
fun MetadataEditor(document: DocumentModel, enabled: Boolean, onSave: (MetadataDraftModel) -> Unit) {
    val saved = MetadataDraftModel(
        title = document.metadata.title.orEmpty(),
        documentDate = document.metadata.documentDate?.toString().orEmpty(),
        correspondent = document.metadata.correspondent.orEmpty(),
    )
    var draft by remember(key1 = document.id, key2 = saved) { mutableStateOf(value = saved) }
    Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
        OutlinedTextField(
            value = draft.title,
            onValueChange = { draft = draft.copy(title = it) },
            label = { Text(text = "Title") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )
        OutlinedTextField(
            value = draft.documentDate,
            onValueChange = { draft = draft.copy(documentDate = it) },
            label = { Text(text = "Document date (YYYY-MM-DD)") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )
        OutlinedTextField(
            value = draft.correspondent,
            onValueChange = { draft = draft.copy(correspondent = it) },
            label = { Text(text = "Correspondent") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )
        Button(onClick = { onSave(draft) }, enabled = enabled && draft != saved) {
            Icon(
                imageVector = Icons.Rounded.Check,
                contentDescription = null,
                modifier = Modifier.size(size = Spacing.icon),
            )
            Spacer(modifier = Modifier.width(width = Spacing.small))
            Text(text = "Save details")
        }
    }
}
