package dms.ui.dashboard

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Timeline
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dms.application.port.state.model.ActivityEntry
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.CardTitle
import dms.ui.theme.Spacing

/** What the tool did recently on this device, newest first. */
@Composable
fun ActivityPanel(entries: List<ActivityEntry>, modifier: Modifier = Modifier) {
    val colors = MaterialTheme.colorScheme
    AppCard(modifier = modifier) {
        CardTitle(title = "Recent activity", icon = Icons.Rounded.Timeline, tint = colors.tertiary)
        if (entries.isEmpty()) {
            Text(text = "Nothing yet", style = MaterialTheme.typography.bodyMedium, color = colors.onSurfaceVariant)
        }
        LazyColumn(verticalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
            items(items = entries) { entry ->
                Row(
                    verticalAlignment = Alignment.Top,
                    horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
                ) {
                    Box(
                        modifier = Modifier
                            .padding(top = Spacing.small)
                            .size(size = 8.dp)
                            .background(color = colors.tertiary, shape = CircleShape),
                    )
                    Column {
                        Text(
                            text = "${ActivityTexts.verb(kind = entry.kind)} ${entry.subject}",
                            style = MaterialTheme.typography.bodyMedium,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                        Text(
                            text = ActivityTexts.time(instant = entry.at),
                            style = MaterialTheme.typography.labelSmall,
                            color = colors.onSurfaceVariant,
                        )
                    }
                }
            }
        }
    }
}
