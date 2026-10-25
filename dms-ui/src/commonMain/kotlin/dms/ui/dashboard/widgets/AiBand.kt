package dms.ui.dashboard.widgets

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.AutoAwesome
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import dms.ui.components.ReasonTooltip
import dms.ui.dashboard.widgets.model.AiBandModel
import dms.ui.theme.AppColors
import dms.ui.theme.Spacing

/**
 * What the AI can do for the library right now, framed in the AI gradient: how many documents miss
 * tags or a category, and a button for each run that has work. Nothing changes before the review.
 */
@Composable
fun AiBand(band: AiBandModel, onClassify: () -> Unit, onTag: () -> Unit) {
    val shape = MaterialTheme.shapes.large
    val gradient = Brush.linearGradient(colors = AppColors.aiGradient)
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .border(width = Spacing.hairline, brush = gradient, shape = shape)
            .background(color = MaterialTheme.colorScheme.surface, shape = shape)
            .padding(horizontal = Spacing.large, vertical = Spacing.medium),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
    ) {
        SparkleBadge(gradient = gradient)
        Column(modifier = Modifier.weight(weight = 1f)) {
            Text(text = band.headline, style = MaterialTheme.typography.titleSmall)
            Text(
                text = "The AI makes suggestions. Nothing changes until you approve each one.",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        band.classifyLabel?.let { label ->
            ReasonTooltip(reason = band.busyReason) {
                OutlinedButton(onClick = onClassify, enabled = band.busyReason == null) {
                    Text(text = label, maxLines = 1, softWrap = false)
                }
            }
        }
        band.tagLabel?.let { label ->
            ReasonTooltip(reason = band.busyReason) {
                GradientButton(label = label, gradient = gradient, enabled = band.busyReason == null, onClick = onTag)
            }
        }
    }
}

@Composable
private fun SparkleBadge(gradient: Brush) {
    Box(
        modifier = Modifier
            .size(size = Spacing.xxlarge)
            .background(brush = gradient, shape = MaterialTheme.shapes.medium),
        contentAlignment = Alignment.Center,
    ) {
        Icon(
            imageVector = Icons.Rounded.AutoAwesome,
            contentDescription = null,
            tint = AppColors.onAiGradient,
            modifier = Modifier.size(size = Spacing.icon),
        )
    }
}

/** The main AI action: white on the AI gradient, which stays readable on every stop. */
@Composable
private fun GradientButton(label: String, gradient: Brush, enabled: Boolean, onClick: () -> Unit) {
    Button(
        onClick = onClick,
        enabled = enabled,
        modifier = Modifier
            .alpha(alpha = if (enabled) 1f else 0.5f)
            .background(brush = gradient, shape = ButtonDefaults.shape),
        colors = ButtonDefaults.buttonColors(
            containerColor = Color.Transparent,
            contentColor = AppColors.onAiGradient,
            disabledContainerColor = Color.Transparent,
            disabledContentColor = AppColors.onAiGradient,
        ),
    ) {
        Icon(
            imageVector = Icons.Rounded.AutoAwesome,
            contentDescription = null,
            modifier = Modifier.size(size = Spacing.icon),
        )
        Spacer(modifier = Modifier.width(width = Spacing.small))
        Text(text = label, maxLines = 1, softWrap = false)
    }
}
