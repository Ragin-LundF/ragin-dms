package dms.ui.dashboard.attention

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.components.ActionButtons
import dms.ui.components.surface.IconBadge
import dms.ui.dashboard.attention.model.AttentionRowModel
import dms.ui.dashboard.attention.model.AttentionSeverity
import dms.ui.dashboard.duplicate.DuplicateCopies
import dms.ui.dashboard.widgets.RowThumbnail
import dms.ui.preview.ThumbnailLoader
import dms.ui.theme.AppColors
import dms.ui.theme.Spacing

/**
 * One row of "Needs your attention": a stripe in the colour of its severity on the left, whose
 * label starts the detail line, a thumbnail, the title and detail, and the row's buttons. Buttons
 * that only open a file come first and always work.
 */
@Composable
fun AttentionRowView(row: AttentionRowModel, busyReason: String?, thumbnails: ThumbnailLoader) {
    val colors = MaterialTheme.colorScheme
    val stripe = severityColor(severity = row.severity)
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .drawBehind {
                val width = Spacing.xsmall.toPx()
                drawRoundRect(
                    color = stripe,
                    size = Size(width = width, height = size.height),
                    cornerRadius = CornerRadius(x = width / 2),
                )
            }
            .padding(start = Spacing.xsmall + Spacing.medium, end = Spacing.large)
            .padding(vertical = Spacing.small),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
    ) {
        RowThumbnail(
            file = row.preview,
            loader = thumbnails,
            onClick = { (row.opens.firstOrNull() ?: row.copies.firstOrNull()?.open)?.onClick?.invoke() },
        ) {
            IconBadge(icon = row.icon, tint = colors.onSurfaceVariant, size = Spacing.xxlarge)
        }
        Column(modifier = Modifier.weight(weight = 1.4f)) {
            Text(
                text = row.title,
                style = MaterialTheme.typography.titleSmall,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                text = listOf(row.severity.label, row.detail)
                    .filter { it.isNotEmpty() }
                    .joinToString(separator = " · "),
                style = MaterialTheme.typography.bodySmall,
                color = colors.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            DuplicateCopies(
                copies = row.copies,
                busyReason = busyReason,
                modifier = Modifier.padding(top = Spacing.small),
            )
        }
        // The buttons wrap within their share, so a long label never pushes the title out.
        ActionButtons(
            buttons = row.opens + row.buttons,
            enabled = true,
            busyReason = null,
            modifier = Modifier.weight(weight = 1f, fill = false),
        )
    }
}

@Composable
private fun severityColor(severity: AttentionSeverity): Color {
    return when (severity) {
        AttentionSeverity.PASSED -> MaterialTheme.colorScheme.error
        AttentionSeverity.SOON -> AppColors.statusWarning
        AttentionSeverity.INFO -> MaterialTheme.colorScheme.primary
    }
}
