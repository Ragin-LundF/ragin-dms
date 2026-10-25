package dms.ui.document.suggestion

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.AutoAwesome
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ButtonStyle
import dms.ui.components.surface.LabelChip
import dms.ui.theme.AppColors
import dms.ui.theme.Spacing

/**
 * A suggestion of the AI that waits for this document's review, in the AI's gradient frame: its
 * [values] as chips, "Review…" to open the review and "Dismiss" to reject it, which waits for a busy
 * library ([busyReason]).
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun SuggestionCard(values: List<String>, busyReason: String?, onReview: () -> Unit, onDismiss: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    val shape = MaterialTheme.shapes.large
    val frame = BorderStroke(width = Spacing.hairline, brush = Brush.linearGradient(colors = AppColors.aiGradient))
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(shape = shape)
            .background(color = colors.surface)
            .border(border = frame, shape = shape)
            .padding(all = Spacing.medium),
        verticalArrangement = Arrangement.spacedBy(space = Spacing.small),
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
        ) {
            Icon(
                imageVector = Icons.Rounded.AutoAwesome,
                contentDescription = null,
                tint = colors.primary,
                modifier = Modifier.size(size = Spacing.icon),
            )
            Text(text = "AI suggestion", style = MaterialTheme.typography.titleSmall)
        }
        FlowRow(
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.xsmall),
            verticalArrangement = Arrangement.spacedBy(space = Spacing.xsmall),
        ) {
            values.forEach { LabelChip(text = it, color = colors.primary) }
        }
        ActionButtons(
            buttons = listOf(
                ActionButtonModel(label = "Review…", style = ButtonStyle.TONAL, onClick = onReview),
                ActionButtonModel(
                    label = "Dismiss",
                    style = ButtonStyle.TEXT,
                    disabledReason = busyReason,
                    onClick = onDismiss,
                ),
            ),
            enabled = true,
            busyReason = null,
        )
    }
}
