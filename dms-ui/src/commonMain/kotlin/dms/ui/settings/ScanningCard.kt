package dms.ui.settings

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.TravelExplore
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.ui.components.ReasonTooltip
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.CardTitle
import dms.ui.theme.Spacing

/** How a scan treats files the catalog does not know yet. */
@Composable
fun ScanningCard(addNewFiles: Boolean, busyReason: String?, onAddNewFilesChange: (Boolean) -> Unit) {
    AppCard(modifier = Modifier.widthIn(max = 920.dp)) {
        CardTitle(
            title = "Scanning",
            icon = Icons.Rounded.TravelExplore,
            tint = MaterialTheme.colorScheme.tertiary,
            subtitle = "What happens with files the library does not know yet",
        )
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.large),
        ) {
            Column(modifier = Modifier.weight(weight = 1f)) {
                Text(text = "Add new files while scanning", style = MaterialTheme.typography.titleSmall)
                Text(
                    text = if (addNewFiles) {
                        "New files appear in the library as soon as the scan has read them. " +
                            "Files stay where they are; only the catalog learns about them."
                    } else {
                        "New files are listed on the dashboard, and you adopt them one by one or all at once."
                    },
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            ReasonTooltip(reason = busyReason) {
                Switch(checked = addNewFiles, onCheckedChange = onAddNewFilesChange, enabled = busyReason == null)
            }
        }
    }
}
