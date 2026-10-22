package dms.ui.restructure

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.DriveFileMove
import androidx.compose.material3.Checkbox
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ButtonStyle
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.CardTitle
import dms.ui.restructure.model.MoveGroupModel
import dms.ui.restructure.model.RestructureUiStateModel
import dms.ui.theme.Spacing

/**
 * The planned restructure: the new folders with how many documents go into each (untick a folder to
 * leave its documents where they are), what stays and why, the names that get a suffix, and Apply.
 */
@Composable
fun PlanCard(state: RestructureUiStateModel, viewModel: RestructureViewModel) {
    AppCard(modifier = Modifier.fillMaxWidth()) {
        CardTitle(
            title = "Plan",
            icon = Icons.Rounded.DriveFileMove,
            tint = MaterialTheme.colorScheme.secondary,
            subtitle = "${state.includedMoves} moves into ${state.groups.count { it.included }} folders · " +
                "${state.unchanged} already in place",
        )
        state.skipped.takeIf { it.isNotEmpty() }?.let {
            Note(text = "Left where they are: ${it.joinToString(separator = ", ")}")
        }
        state.renamed.takeIf { it.isNotEmpty() }?.let {
            Note(text = "Name taken, numbered: ${it.joinToString(separator = "; ")}")
        }
        LazyColumn(modifier = Modifier.heightIn(max = 480.dp)) {
            items(items = state.groups, key = { it.folder }) { group ->
                GroupRow(group = group, onToggle = { viewModel.toggleGroup(folder = group.folder) })
            }
        }
        val busyReason = state.busy?.let { "Wait until \"$it\" finishes" }
        ActionButtons(
            buttons = listOf(
                ActionButtonModel(
                    label = "Apply ${state.includedMoves} moves",
                    style = ButtonStyle.FILLED,
                    disabledReason = "Nothing to move".takeIf { state.includedMoves == 0 },
                    onClick = viewModel::apply,
                ),
            ),
            enabled = busyReason == null,
            busyReason = busyReason,
        )
    }
}

@Composable
private fun GroupRow(group: MoveGroupModel, onToggle: () -> Unit) {
    Row(verticalAlignment = Alignment.Top, horizontalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
        Checkbox(checked = group.included, onCheckedChange = { onToggle() })
        Column(modifier = Modifier.weight(weight = 1f)) {
            Text(
                text = "${group.folder.ifEmpty { "Top level" }}  (${group.count})",
                style = MaterialTheme.typography.titleSmall,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            group.samples.forEach { sample ->
                Text(
                    text = sample,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
    }
}

@Composable
private fun Note(text: String) {
    Text(
        text = text,
        style = MaterialTheme.typography.bodySmall,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier = Modifier.fillMaxWidth(),
    )
}
