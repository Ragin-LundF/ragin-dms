package dms.ui.inbox

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.DriveFileMove
import androidx.compose.material.icons.rounded.AutoAwesome
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ButtonStyle
import dms.ui.components.surface.AppCard
import dms.ui.inbox.model.InboxRowModel
import dms.ui.theme.Spacing

/** One inbox document with its actions: review the AI's suggestion or analyze it, or move it by hand. */
@Composable
fun InboxRowView(
    row: InboxRowModel,
    aiAnalysis: Boolean,
    busyReason: String?,
    aiBusyReason: String?,
    onAnalyze: () -> Unit,
    onReview: () -> Unit,
    onMove: () -> Unit,
) {
    AppCard(modifier = Modifier.fillMaxWidth(), contentPadding = Spacing.large) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.large),
        ) {
            Column(modifier = Modifier.weight(weight = 1f)) {
                Text(
                    text = row.name,
                    style = MaterialTheme.typography.titleSmall,
                    maxLines = 1,
                    softWrap = false,
                    overflow = TextOverflow.Ellipsis,
                )
                val where = listOfNotNull(row.subfolder.ifEmpty { null }, row.size, "arrived ${row.added}")
                Text(
                    text = where.joinToString(separator = " · ") + suggestionNote(row = row),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            val move = ActionButtonModel(
                label = "Move to…",
                icon = Icons.AutoMirrored.Rounded.DriveFileMove,
                disabledReason = busyReason,
                onClick = onMove,
            )
            val ai = aiButtons(
                row = row,
                aiAnalysis = aiAnalysis,
                aiBusyReason = aiBusyReason,
                onAnalyze = onAnalyze,
                onReview = onReview,
            )
            ActionButtons(buttons = ai + move, enabled = true, busyReason = null)
        }
    }
}

private fun suggestionNote(row: InboxRowModel): String {
    val target = row.suggestion?.targetFolder?.path?.value
    return when {
        row.suggestion == null -> ""
        target != null -> " · AI suggests $target"
        else -> " · AI suggestion waiting"
    }
}

private fun aiButtons(
    row: InboxRowModel,
    aiAnalysis: Boolean,
    aiBusyReason: String?,
    onAnalyze: () -> Unit,
    onReview: () -> Unit,
): List<ActionButtonModel> {
    if (!aiAnalysis) {
        return emptyList()
    }
    val suggestion = row.suggestion
    if (suggestion != null) {
        return listOf(
            ActionButtonModel(
                label = "Review",
                icon = Icons.Rounded.AutoAwesome,
                style = ButtonStyle.TONAL,
                onClick = onReview,
            ),
        )
    }
    return listOf(
        ActionButtonModel(
            label = "Analyze",
            icon = Icons.Rounded.AutoAwesome,
            style = ButtonStyle.TONAL,
            disabledReason = if (row.analyzable) aiBusyReason else "The AI does not read this kind of file",
            onClick = onAnalyze,
        ),
    )
}
