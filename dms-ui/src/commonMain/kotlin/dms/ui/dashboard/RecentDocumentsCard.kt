package dms.ui.dashboard

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.History
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.style.TextOverflow
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.ui.components.FullViewButton
import dms.ui.components.ReasonTooltip
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.CardTitle
import dms.ui.components.surface.KindBadge
import dms.ui.dashboard.model.TaskFileModel
import dms.ui.dashboard.stats.model.RecentDocumentModel
import dms.ui.dashboard.widgets.RowThumbnail
import dms.ui.preview.ThumbnailLoader
import dms.ui.theme.Spacing

/**
 * The documents added last, each with a thumbnail from [primaryFolderId] and when it was added; a
 * click opens one in the library, and the card opens all of them there.
 */
@Composable
fun RecentDocumentsCard(
    documents: List<RecentDocumentModel>,
    primaryFolderId: LibraryFolderIdModel?,
    thumbnails: ThumbnailLoader,
    onOpen: (DocumentIdModel) -> Unit,
    onShowAll: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = MaterialTheme.colorScheme
    AppCard(modifier = modifier) {
        CardTitle(title = "Recently added", icon = Icons.Rounded.History, tint = colors.primary) {
            FullViewButton(label = "Show all recently added documents", onClick = onShowAll)
        }
        if (documents.isEmpty()) {
            Text(
                text = "Drop files anywhere on this page to add them",
                style = MaterialTheme.typography.bodyMedium,
                color = colors.onSurfaceVariant,
            )
        }
        documents.forEach { document ->
            RecentRow(
                document = document,
                file = primaryFolderId?.let { TaskFileModel(label = "Open", folderId = it, path = document.path) },
                thumbnails = thumbnails,
                onOpen = { onOpen(document.id) },
            )
        }
    }
}

@Composable
private fun RecentRow(
    document: RecentDocumentModel,
    file: TaskFileModel?,
    thumbnails: ThumbnailLoader,
    onOpen: () -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(shape = MaterialTheme.shapes.small)
            .clickable(onClick = onOpen)
            .padding(vertical = Spacing.xsmall, horizontal = Spacing.xsmall),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
    ) {
        RowThumbnail(file = file, loader = thumbnails, onClick = onOpen) {
            KindBadge(kind = document.kind, size = Spacing.xxlarge)
        }
        Column(modifier = Modifier.weight(weight = 1f)) {
            Text(
                text = document.title,
                style = MaterialTheme.typography.bodyMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                text = listOfNotNull(document.folder.ifEmpty { "Top level" }, document.categoryName)
                    .joinToString(separator = " · "),
                style = MaterialTheme.typography.bodySmall,
                color = colors.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        ReasonTooltip(reason = document.addedOn) {
            Text(
                text = document.added,
                style = MaterialTheme.typography.labelMedium,
                color = colors.onSurfaceVariant,
                maxLines = 1,
                softWrap = false,
            )
        }
    }
}
