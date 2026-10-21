package dms.ui.dashboard.model

import dms.application.port.state.model.ActivityEntry
import dms.ui.dashboard.stats.model.LibraryStats
import java.time.Instant

/** Everything the dashboard shows; derived from the session and the last dashboard build. */
data class DashboardUiState(
    val folders: List<FolderStatus> = emptyList(),
    val documentCount: Int = 0,
    val uncategorizedCount: Int = 0,
    val groups: List<TaskGroupState> = emptyList(),
    val filter: TaskGroup? = null,
    val activity: List<ActivityEntry> = emptyList(),
    val lastScanAt: Instant? = null,
    val busy: String? = null,
    val stats: LibraryStats = LibraryStats(),
) {
    val openTaskCount: Int
        get() {
            return groups.sumOf { it.rows.size }
        }
}
