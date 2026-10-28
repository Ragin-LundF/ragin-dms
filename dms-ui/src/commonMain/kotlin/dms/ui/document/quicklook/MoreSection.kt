package dms.ui.document.quicklook

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.ChevronRight
import androidx.compose.material.icons.rounded.ExpandMore
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentModel
import dms.ui.document.QuickLookViewModel
import dms.ui.document.details.DetailsSections
import dms.ui.document.details.NotesAndLinksSections
import dms.ui.pdf.PdfSection
import dms.ui.theme.Spacing

/**
 * The rarely used parts, folded so the first view stays short: note and links, fields, rule and
 * retention, and the PDF tools. It stays open or folded while the quick look moves on.
 */
@Composable
fun MoreSection(document: DocumentModel, catalog: CatalogModel, busyReason: String?, viewModel: QuickLookViewModel) {
    var open by remember { mutableStateOf(value = false) }
    val aiEnabled by viewModel.aiEnabled.collectAsState()
    Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.medium)) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .clip(shape = MaterialTheme.shapes.extraSmall)
                .clickable(onClickLabel = if (open) "Fold" else "Unfold") { open = !open }
                .padding(vertical = Spacing.xsmall),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.xsmall),
        ) {
            Icon(
                imageVector = if (open) Icons.Rounded.ExpandMore else Icons.Rounded.ChevronRight,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.size(size = Spacing.icon),
            )
            Text(text = "Notes, links, retention, pages", style = MaterialTheme.typography.titleSmall)
        }
        if (open) {
            NotesAndLinksSections(
                document = document,
                catalog = catalog,
                busyReason = busyReason,
                controller = viewModel.details,
                onOpenDocument = viewModel::open,
            )
            DetailsSections(
                document = document,
                catalog = catalog,
                busyReason = busyReason,
                controller = viewModel.details,
            )
            PdfSection(
                document = document,
                busyReason = busyReason,
                pages = viewModel.pages,
                actions = viewModel.pdf,
                rebuild = viewModel.rebuild,
                aiEnabled = aiEnabled,
            )
        }
    }
}
