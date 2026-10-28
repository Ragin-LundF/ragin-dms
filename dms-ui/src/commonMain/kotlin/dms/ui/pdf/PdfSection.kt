package dms.ui.pdf

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.AutoFixHigh
import androidx.compose.material.icons.rounded.DocumentScanner
import androidx.compose.material.icons.rounded.Layers
import androidx.compose.material.icons.rounded.MergeType
import androidx.compose.material.icons.rounded.TextSnippet
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import dms.domain.document.DocumentKind
import dms.domain.document.model.DocumentModel
import dms.ui.components.ActionButtons
import dms.ui.components.SectionHeader
import dms.ui.components.model.ActionButtonModel
import dms.ui.pdf.rebuild.RebuildController

/**
 * The PDF tools of the quick look: edit the pages, merge with other PDFs, make a scan searchable, and
 * convert a scan without text to a real PDF, which needs the AI ([aiEnabled]).
 */
@Composable
fun PdfSection(
    document: DocumentModel,
    busyReason: String?,
    pages: PageEditorController,
    actions: PdfActions,
    rebuild: RebuildController,
    aiEnabled: Boolean,
) {
    if (document.kind != DocumentKind.PDF) {
        return
    }
    val ocrAvailable by actions.ocrAvailable.collectAsState()
    val rebuildable by rebuild.available.collectAsState()
    val aiBusyReason by rebuild.aiBusyReason.collectAsState()
    LaunchedEffect(key1 = document.id, key2 = document.contentHash) { rebuild.check(document = document) }
    val convert = ActionButtonModel(
        label = "Convert to real PDF",
        icon = Icons.Rounded.AutoFixHigh,
        disabledReason = aiBusyReason,
    ) { rebuild.start(document = document) }
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
        ) + listOfNotNull(convert.takeIf { aiEnabled && document.id in rebuildable }),
        enabled = busyReason == null,
        busyReason = busyReason,
    )
}
