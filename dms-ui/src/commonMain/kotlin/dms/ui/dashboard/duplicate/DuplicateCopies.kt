package dms.ui.dashboard.duplicate

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.components.ReasonTooltip
import dms.ui.components.StyledButton
import dms.ui.dashboard.model.DuplicateCopyModel
import dms.ui.theme.Spacing

/**
 * The copies of a duplicates task, one line each: the path and what is assigned to it, and beside them
 * its Open and Keep buttons side by side, so the buttons of all copies line up in one column. Opening
 * stays possible while operations run. Shows nothing when there are no copies.
 */
@Composable
fun DuplicateCopies(copies: List<DuplicateCopyModel>, busyReason: String?, modifier: Modifier = Modifier) {
    if (copies.isEmpty()) {
        return
    }
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
        copies.forEach { copy ->
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
            ) {
                Column(modifier = Modifier.weight(weight = 1f)) {
                    Text(
                        text = copy.path,
                        style = MaterialTheme.typography.bodyMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    Text(
                        text = copy.detail,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
                ) {
                    StyledButton(button = copy.open, enabled = true)
                    val reason = copy.keep.disabledReason ?: busyReason
                    ReasonTooltip(reason = reason) {
                        StyledButton(button = copy.keep, enabled = reason == null)
                    }
                }
            }
        }
    }
}
