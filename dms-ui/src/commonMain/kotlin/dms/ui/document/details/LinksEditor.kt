package dms.ui.document.details

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.OpenInNew
import androidx.compose.material.icons.rounded.AddLink
import androidx.compose.material.icons.rounded.LinkOff
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import dms.domain.document.model.DocumentIdModel
import dms.ui.components.ActionButtons
import dms.ui.components.ReasonTooltip
import dms.ui.components.model.ActionButtonModel
import dms.ui.document.details.model.LinkCandidateModel
import dms.ui.theme.Spacing

/** The documents that belong to this one, each to open or unlink, and "Link to…" for another. */
@Composable
fun LinksEditor(
    links: List<LinkCandidateModel>,
    busyReason: String?,
    onOpen: (DocumentIdModel) -> Unit,
    onUnlink: (DocumentIdModel) -> Unit,
    onAdd: () -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.xsmall)) {
        if (links.isEmpty()) {
            Text(
                text = "No linked documents",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        links.forEach { link ->
            Row(verticalAlignment = Alignment.CenterVertically) {
                Column(modifier = Modifier.weight(weight = 1f)) {
                    Text(text = link.name, maxLines = 1, softWrap = false, overflow = TextOverflow.Ellipsis)
                    Text(
                        text = listOfNotNull(link.label, link.path).joinToString(separator = " · "),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
                IconButton(onClick = { onOpen(link.documentId) }) {
                    Icon(imageVector = Icons.AutoMirrored.Rounded.OpenInNew, contentDescription = "Open ${link.name}")
                }
                ReasonTooltip(reason = busyReason) {
                    IconButton(onClick = { onUnlink(link.documentId) }, enabled = busyReason == null) {
                        Icon(imageVector = Icons.Rounded.LinkOff, contentDescription = "Unlink ${link.name}")
                    }
                }
            }
        }
        ActionButtons(
            buttons = listOf(ActionButtonModel(label = "Link to…", icon = Icons.Rounded.AddLink, onClick = onAdd)),
            enabled = busyReason == null,
            busyReason = busyReason,
        )
    }
}
