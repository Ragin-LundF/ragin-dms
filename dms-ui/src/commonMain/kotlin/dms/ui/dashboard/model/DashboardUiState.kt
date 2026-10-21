package dms.ui.dashboard.model

import dms.application.port.state.model.ActivityEntry
import dms.ui.dashboard.stats.model.LibraryStats
import java.time.Instant

/**
 * Everything the dashboard shows; derived from the session and the last dashboard build.
 * [classifyCandidates] and [tagCandidates] count the documents the AI can read that have no
 * classification or no tags, and no suggestion waiting.
 */
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
    val classifyCandidates: Int = 0,
    val tagCandidates: Int = 0,
) {
    val openTaskCount: Int
        get() {
            return groups.sumOf { it.rows.size }
        }
}
