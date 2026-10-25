package dms.ui.dashboard.section

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import dms.domain.document.model.DocumentIdModel
import dms.ui.dashboard.attention.AttentionPanel
import dms.ui.dashboard.attention.model.AttentionListModel
import dms.ui.dashboard.RecentDocumentsCard
import dms.ui.dashboard.model.DashboardUiStateModel
import dms.ui.preview.ThumbnailLoader
import dms.ui.theme.Spacing

/** What needs the user, beside what came in last. */
@Composable
fun FocusSection(
    state: DashboardUiStateModel,
    attention: AttentionListModel,
    busyReason: String?,
    thumbnails: ThumbnailLoader,
    onShowAllTasks: () -> Unit,
    onOpenDocument: (DocumentIdModel) -> Unit,
    onOpenRecent: () -> Unit,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.Top,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.large),
    ) {
        AttentionPanel(
            attention = attention,
            busyReason = busyReason,
            thumbnails = thumbnails,
            onShowAllTasks = onShowAllTasks,
            modifier = Modifier.weight(weight = 1.35f),
        )
        RecentDocumentsCard(
            documents = state.stats.recent,
            primaryFolderId = state.primaryFolderId,
            thumbnails = thumbnails,
            onOpen = onOpenDocument,
            onShowAll = onOpenRecent,
            modifier = Modifier.weight(weight = 1f),
        )
    }
}
