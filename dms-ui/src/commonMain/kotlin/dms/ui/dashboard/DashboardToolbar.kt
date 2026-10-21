package dms.ui.dashboard

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import dms.ui.components.ActionButtons
import dms.ui.components.DialogController
import dms.ui.components.model.ActionButton
import dms.ui.theme.Spacing
import java.time.Instant

/** The title and the actions that concern the whole library. */
@Composable
fun DashboardToolbar(
    lastScanAt: Instant?,
    busyReason: String?,
    onScan: () -> Unit,
    onSync: () -> Unit,
    onMerge: () -> Unit,
) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(bottom = Spacing.medium),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
    ) {
        Text(text = "Dashboard", style = MaterialTheme.typography.headlineSmall)
        Spacer(modifier = Modifier.weight(weight = 1f))
        Text(
            text = lastScanAt?.let { "Last scan ${ActivityTexts.time(instant = it)}" } ?: "Not scanned yet",
            style = MaterialTheme.typography.bodySmall,
        )
        ActionButtons(
            buttons = listOf(
                ActionButton(label = "Scan now", onClick = onScan),
                ActionButton(label = "Sync to all destinations", onClick = onSync),
                ActionButton(label = "Merge all folders", onClick = onMerge),
            ),
            enabled = busyReason == null,
            busyReason = busyReason,
        )
    }
}
