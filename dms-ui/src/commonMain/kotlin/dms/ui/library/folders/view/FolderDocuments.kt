package dms.ui.library.folders.view

import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.ui.PlatformActions
import dms.ui.components.layout.FileDropZone
import dms.ui.library.folders.FolderTexts
import dms.ui.library.folders.model.FolderBrowserStateModel
import dms.ui.theme.Spacing

/**
 * The "Documents" section: the documents directly in the open folder, drawn by [documents], or a drop
 * zone saying there are none. Files dropped from the file manager go to [onDropFiles] with the open folder.
 */
@Composable
fun FolderDocuments(
    state: FolderBrowserStateModel,
    platform: PlatformActions,
    onDropFiles: (List<String>, RelativePathModel) -> Unit,
    modifier: Modifier = Modifier,
    documents: @Composable (List<DocumentModel>) -> Unit,
) {
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
        FolderSectionTitle(
            title = "Documents",
            count = state.documents.size,
            hint = "Drag documents onto a folder, the tree or the path above to move them",
        )
        FileDropZone(
            platform = platform,
            label = "Add to ${FolderTexts.where(folder = state.open)}",
            onDrop = { onDropFiles(it, state.open) },
            modifier = Modifier.weight(weight = 1f).fillMaxWidth(),
        ) {
            if (state.documents.isEmpty()) {
                EmptyDropZone(open = state.open)
            } else {
                documents(state.documents)
            }
        }
    }
}

@Composable
private fun EmptyDropZone(open: RelativePathModel) {
    val colors = MaterialTheme.colorScheme
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .border(width = Spacing.hairline, color = colors.outlineVariant, shape = MaterialTheme.shapes.medium)
            .padding(all = Spacing.xlarge),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = FolderTexts.emptyDocuments(open = open),
            style = MaterialTheme.typography.bodyMedium,
            color = colors.onSurfaceVariant,
            textAlign = TextAlign.Center,
        )
    }
}
