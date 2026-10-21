package dms.ui.dashboard.widgets

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dms.ui.components.chart.growIn
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.IconBadge
import dms.ui.theme.Spacing

/** One headline figure: an icon, the figure, what it means, and optionally how far along it is. */
@Composable
fun StatCard(
    title: String,
    value: String,
    caption: String,
    icon: ImageVector,
    tint: Color,
    modifier: Modifier = Modifier,
    progress: Float? = null,
    onClick: (() -> Unit)? = null,
) {
    AppCard(modifier = modifier, onClick = onClick, contentPadding = Spacing.large + Spacing.xsmall) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
        ) {
            IconBadge(icon = icon, tint = tint, size = 44.dp)
            Column {
                Text(
                    text = title,
                    style = MaterialTheme.typography.labelLarge,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    softWrap = false,
                )
                Text(text = value, style = MaterialTheme.typography.headlineMedium, maxLines = 1, softWrap = false)
            }
        }
        Text(
            text = caption,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.padding(top = Spacing.medium),
        )
        progress?.let { target ->
            val grown = growIn(target = target)
            LinearProgressIndicator(
                progress = { grown },
                modifier = Modifier.fillMaxWidth().padding(top = Spacing.small).height(height = 6.dp),
                color = tint,
                trackColor = MaterialTheme.colorScheme.surfaceContainerHigh,
                strokeCap = StrokeCap.Round,
            )
        }
    }
}
