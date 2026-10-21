package dms.ui.dashboard.duplicate

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.components.ActionButtons
import dms.ui.dashboard.model.DuplicateCopy
import dms.ui.theme.Spacing

/**
 * The copies of a duplicates task, one line each: the path and what is assigned to it, then its Open
 * and Keep buttons, so the buttons of all copies line up. Opening stays possible while operations run.
 * Shows nothing when there are no copies.
 */
@Composable
fun DuplicateCopies(copies: List<DuplicateCopy>, busyReason: String?, modifier: Modifier = Modifier) {
    if (copies.isEmpty()) {
        return
    }
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(space = Spacing.xsmall)) {
        copies.forEach { copy ->
            Row(
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
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
                ActionButtons(buttons = listOf(copy.open), enabled = true, busyReason = null)
                ActionButtons(buttons = listOf(copy.keep), enabled = busyReason == null, busyReason = busyReason)
            }
        }
    }
}
