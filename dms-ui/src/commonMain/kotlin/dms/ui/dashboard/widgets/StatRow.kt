package dms.ui.dashboard.widgets

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dms.ui.components.chart.Sparkline
import dms.ui.components.chart.growIn
import dms.ui.components.surface.AppCard
import dms.ui.dashboard.stats.model.StatColumnModel
import dms.ui.dashboard.stats.model.StatLink
import dms.ui.theme.Spacing

/** The headline figures in one card, side by side between hairlines; a figure with a link opens it. */
@Composable
fun StatRow(columns: List<StatColumnModel>, onOpen: (StatLink) -> Unit) {
    AppCard(contentPadding = 0.dp) {
        Row(modifier = Modifier.fillMaxWidth().height(intrinsicSize = IntrinsicSize.Min)) {
            columns.forEachIndexed { index, column ->
                if (index > 0) {
                    VerticalDivider(thickness = Spacing.hairline, color = MaterialTheme.colorScheme.outlineVariant)
                }
                StatColumn(
                    column = column,
                    onClick = column.link?.let { link -> { onOpen(link) } },
                    modifier = Modifier.weight(weight = 1f).fillMaxHeight(),
                )
            }
        }
    }
}

@Composable
private fun StatColumn(column: StatColumnModel, onClick: (() -> Unit)?, modifier: Modifier) {
    val colors = MaterialTheme.colorScheme
    Column(
        modifier = modifier
            .let { if (onClick == null) it else it.clickable(onClick = onClick) }
            .padding(horizontal = Spacing.large, vertical = Spacing.medium),
        verticalArrangement = Arrangement.spacedBy(space = Spacing.hairline * 2),
    ) {
        Text(
            text = column.label,
            style = MaterialTheme.typography.labelMedium,
            color = colors.onSurfaceVariant,
            maxLines = 1,
            softWrap = false,
        )
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
        ) {
            Text(text = column.value, style = MaterialTheme.typography.headlineMedium, maxLines = 1, softWrap = false)
            if (column.trend.size > 1) {
                Sparkline(
                    values = column.trend,
                    color = colors.primary,
                    modifier = Modifier.size(width = Spacing.xxlarge * 2, height = Spacing.xlarge),
                )
            }
        }
        column.meter?.let { target ->
            val grown = growIn(target = target)
            LinearProgressIndicator(
                progress = { grown },
                modifier = Modifier.fillMaxWidth().padding(vertical = Spacing.xsmall).height(height = Spacing.xsmall),
                color = colors.primary,
                trackColor = colors.surfaceContainerHigh,
                strokeCap = StrokeCap.Round,
                gapSize = 0.dp,
                drawStopIndicator = {},
            )
        }
        Text(
            text = if (onClick == null) column.caption else "${column.caption} →",
            style = MaterialTheme.typography.bodySmall,
            color = colors.onSurfaceVariant,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}
