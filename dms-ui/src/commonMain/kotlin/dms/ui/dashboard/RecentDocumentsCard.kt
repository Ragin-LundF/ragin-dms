package dms.ui.dashboard

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
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
import dms.ui.components.FullViewButton
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.CardTitle
import dms.ui.components.surface.KindBadge
import dms.ui.dashboard.panel.ActivityTexts
import dms.ui.dashboard.stats.model.RecentDocumentModel
import dms.ui.theme.Spacing

/** The documents added last; a click opens one in the library, and the card opens all of them there. */
@Composable
fun RecentDocumentsCard(
    documents: List<RecentDocumentModel>,
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
        Column(modifier = Modifier.verticalScroll(state = rememberScrollState())) {
            documents.forEach { document ->
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(shape = MaterialTheme.shapes.small)
                        .clickable { onOpen(document.id) }
                        .padding(vertical = Spacing.small, horizontal = Spacing.xsmall),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
                ) {
                    KindBadge(kind = document.kind)
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
                    Text(
                        text = ActivityTexts.time(instant = document.addedAt),
                        style = MaterialTheme.typography.labelSmall,
                        color = colors.onSurfaceVariant,
                    )
                }
            }
        }
    }
}
