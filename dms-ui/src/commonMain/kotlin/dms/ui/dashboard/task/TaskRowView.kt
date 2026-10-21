package dms.ui.dashboard.task

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedCard
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import dms.ui.components.ActionButtons
import dms.ui.dashboard.model.TaskRowState
import dms.ui.theme.Spacing

/** One open task with its description and actions. */
@Composable
fun TaskRowView(row: TaskRowState, busyReason: String?) {
    OutlinedCard(modifier = Modifier.fillMaxWidth().padding(bottom = Spacing.small)) {
        Column(modifier = Modifier.padding(all = Spacing.medium)) {
            Text(text = row.title, maxLines = 1, softWrap = false, overflow = TextOverflow.Ellipsis)
            if (row.detail.isNotEmpty()) {
                Text(
                    text = row.detail,
                    style = MaterialTheme.typography.bodySmall,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            Column(modifier = Modifier.padding(top = Spacing.small)) {
                ActionButtons(buttons = row.buttons, enabled = busyReason == null, busyReason = busyReason)
            }
        }
    }
}
