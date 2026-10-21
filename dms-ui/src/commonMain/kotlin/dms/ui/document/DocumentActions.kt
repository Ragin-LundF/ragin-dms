package dms.ui.document

import androidx.compose.runtime.Composable
import dms.domain.document.Document
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButton

/** Rename and move the document, which waits for a busy library, and open or show it, which never waits. */
@Composable
fun DocumentActions(document: Document, busyReason: String?, viewModel: InspectorViewModel) {
    ActionButtons(
        buttons = listOf(
            ActionButton(label = "Rename…") { viewModel.rename(document = document) },
            ActionButton(label = "Move…") { viewModel.moveToFolder(document = document) },
        ),
        enabled = busyReason == null,
        busyReason = busyReason,
    )
    ActionButtons(
        buttons = listOf(
            ActionButton(label = "Open") { viewModel.open(document = document) },
            ActionButton(label = "Show in ${viewModel.fileManagerName}") {
                viewModel.showInFileManager(document = document)
            },
        ),
        enabled = true,
        busyReason = null,
    )
}
