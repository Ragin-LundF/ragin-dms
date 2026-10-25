package dms.ui.dashboard.section

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.ui.dashboard.stats.model.LibraryStatsModel
import dms.ui.dashboard.widgets.CategoryCard
import dms.ui.dashboard.widgets.TagCloudCard
import dms.ui.dashboard.widgets.YearChartCard
import dms.ui.theme.Spacing

/** The ways into the library by what documents are about: categories, tags and years. */
@Composable
fun QuickAccessSection(
    stats: LibraryStatsModel,
    onOpenCategory: (String) -> Unit,
    onOpenUncategorized: () -> Unit,
    onOpenTag: (String) -> Unit,
    onOpenYear: (String) -> Unit,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.large),
    ) {
        val card = Modifier.height(height = 340.dp).weight(weight = 1f)
        CategoryCard(
            points = stats.categories,
            uncategorizedCount = stats.uncategorizedCount,
            onOpenCategory = onOpenCategory,
            onOpenUncategorized = onOpenUncategorized,
            modifier = card,
        )
        TagCloudCard(points = stats.tags, onOpenTag = onOpenTag, modifier = card)
        YearChartCard(points = stats.byYear, onOpenYear = onOpenYear, modifier = card)
    }
}
