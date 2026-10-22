package dms.ui.pdf

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.DocumentScanner
import androidx.compose.material.icons.rounded.Layers
import androidx.compose.material.icons.rounded.MergeType
import androidx.compose.material.icons.rounded.TextSnippet
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import dms.domain.document.DocumentKind
import dms.domain.document.model.DocumentModel
import dms.ui.components.ActionButtons
import dms.ui.components.SectionHeader
import dms.ui.components.model.ActionButtonModel

/** The PDF tools of the inspector: edit the pages, merge with other PDFs, make a scan searchable. */
@Composable
fun PdfSection(
    document: DocumentModel,
    busyReason: String?,
    pages: PageEditorController,
    actions: PdfActions,
) {
    if (document.kind != DocumentKind.PDF) {
        return
    }
    val ocrAvailable by actions.ocrAvailable.collectAsState()
    SectionHeader(title = "PDF")
    ActionButtons(
        buttons = listOf(
            ActionButtonModel(label = "Edit pages…", icon = Icons.Rounded.Layers) { pages.open(document = document) },
            ActionButtonModel(label = "Merge with…", icon = Icons.Rounded.MergeType) {
                actions.startMerge(document = document)
            },
            ActionButtonModel(label = "Make searchable", icon = Icons.Rounded.TextSnippet) {
                actions.makeSearchable(document = document, precise = false)
            },
            ActionButtonModel(
                label = "Precise OCR",
                icon = Icons.Rounded.DocumentScanner,
                disabledReason = "Install ocrmypdf to use it".takeIf { !ocrAvailable },
            ) { actions.makeSearchable(document = document, precise = true) },
        ),
        enabled = busyReason == null,
        busyReason = busyReason,
    )
}
