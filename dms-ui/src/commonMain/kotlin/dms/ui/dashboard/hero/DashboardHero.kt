package dms.ui.dashboard.hero

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.MergeType
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material.icons.rounded.Refresh
import androidx.compose.material.icons.rounded.Sync
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import dms.ui.components.FileSizes
import dms.ui.components.layout.SearchField
import dms.ui.dashboard.panel.ActivityTexts
import dms.ui.dashboard.model.DashboardUiStateModel
import dms.ui.theme.AppColors
import dms.ui.theme.Spacing
import java.time.LocalTime

/** The gradient header: a greeting, the size of the library, a search over all documents and the main actions. */
@Composable
fun DashboardHero(
    state: DashboardUiStateModel,
    busyReason: String?,
    onAddFiles: () -> Unit,
    onScan: () -> Unit,
    onSync: () -> Unit,
    onMerge: () -> Unit,
    onClassifyAll: () -> Unit,
    onTagAll: () -> Unit,
    onSearch: (String) -> Unit,
) {
    var query by remember { mutableStateOf(value = "") }
    val hour = remember { LocalTime.now().hour }
    val greeting = when {
        hour < 12 -> "Good morning"
        hour < 18 -> "Good afternoon"
        else -> "Good evening"
    }
    val scanned = state.lastScanAt?.let { "last scan ${ActivityTexts.time(instant = it)}" } ?: "not scanned yet"
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(shape = MaterialTheme.shapes.extraLarge)
            .background(brush = Brush.linearGradient(colors = AppColors.brandGradient))
            .padding(all = 28.dp),
        verticalArrangement = Arrangement.spacedBy(space = Spacing.xlarge),
    ) {
        Column {
            Text(text = greeting, style = MaterialTheme.typography.headlineLarge, color = Color.White)
            Text(
                text = "${state.stats.documentCount} documents · ${FileSizes.format(bytes = state.stats.totalBytes)}" +
                    " · ${state.folders.size} library folders · $scanned",
                style = MaterialTheme.typography.bodyLarge,
                color = Color.White.copy(alpha = 0.85f),
            )
        }
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
        ) {
            SearchField(
                query = query,
                onQueryChange = { query = it },
                placeholder = "Search all documents and press Enter",
                onSearch = { onSearch(query) },
                modifier = Modifier.weight(weight = 1f).widthIn(max = 560.dp),
            )
            HeroButton(label = "Add files", icon = Icons.Rounded.Add, busyReason = busyReason, onClick = onAddFiles)
            HeroButton(label = "Scan", icon = Icons.Rounded.Refresh, busyReason = busyReason, onClick = onScan)
            HeroButton(label = "Sync to all", icon = Icons.Rounded.Sync, busyReason = busyReason, onClick = onSync)
            HeroButton(
                label = "Merge",
                icon = Icons.AutoMirrored.Rounded.MergeType,
                busyReason = busyReason,
                onClick = onMerge,
            )
        }
        HeroAiButtons(state = state, onTagAll = onTagAll, onClassifyAll = onClassifyAll)
    }
}

