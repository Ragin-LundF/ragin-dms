package dms.ui.dashboard.section

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.ui.dashboard.CorrespondentCard
import dms.ui.dashboard.stats.model.LibraryStatsModel
import dms.ui.dashboard.widgets.KindCard
import dms.ui.theme.Spacing

/** Charts of the library: file types and the main correspondents. */
@Composable
fun InsightsSection(stats: LibraryStatsModel, onSearch: (String) -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.large),
    ) {
        val card = Modifier.height(height = 340.dp)
        KindCard(kinds = stats.byKind, modifier = card.weight(weight = 1.3f))
        CorrespondentCard(points = stats.correspondents, onSearch = onSearch, modifier = card.weight(weight = 1f))
    }
}
