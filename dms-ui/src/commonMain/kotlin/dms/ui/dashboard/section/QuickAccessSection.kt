package dms.ui.dashboard.section

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.domain.document.model.DocumentIdModel
import dms.ui.dashboard.RecentDocumentsCard
import dms.ui.dashboard.stats.model.LibraryStatsModel
import dms.ui.dashboard.widgets.CategoryCard
import dms.ui.dashboard.widgets.TagCloudCard
import dms.ui.theme.Spacing

/** The fastest ways into the library: recent documents, categories and tags. */
@Composable
fun QuickAccessSection(
    stats: LibraryStatsModel,
    onOpenDocument: (DocumentIdModel) -> Unit,
    onOpenRecent: () -> Unit,
    onOpenCategory: (String) -> Unit,
    onOpenUncategorized: () -> Unit,
    onOpenTag: (String) -> Unit,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.large),
    ) {
        val card = Modifier.height(height = 360.dp)
        RecentDocumentsCard(
            documents = stats.recent,
            onOpen = onOpenDocument,
            onShowAll = onOpenRecent,
            modifier = card.weight(weight = 1.2f),
        )
        CategoryCard(
            points = stats.categories,
            uncategorizedCount = stats.uncategorizedCount,
            onOpenCategory = onOpenCategory,
            onOpenUncategorized = onOpenUncategorized,
            modifier = card.weight(weight = 1f),
        )
        TagCloudCard(points = stats.tags, onOpenTag = onOpenTag, modifier = card.weight(weight = 1f))
    }
}
