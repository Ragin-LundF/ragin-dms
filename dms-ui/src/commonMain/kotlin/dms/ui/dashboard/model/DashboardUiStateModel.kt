package dms.ui.dashboard.model

import dms.application.port.state.model.ActivityEntryModel
import dms.ui.dashboard.organize.model.InboxSummaryModel
import dms.ui.dashboard.organize.model.UpcomingRowModel
import dms.ui.dashboard.stats.model.LibraryStatsModel
import java.time.Instant

/**
 * Everything the dashboard shows; derived from the session and the last dashboard build.
 * [classifyCandidates] and [tagCandidates] count the documents the AI can read that have no
 * classification or no tags, and no suggestion waiting. [busy] names the library operation that
 * runs now, [aiBusy] the AI run. [inbox] is set while the inbox is switched on.
 */
data class DashboardUiStateModel(
    val folders: List<FolderStatusModel> = emptyList(),
    val documentCount: Int = 0,
    val uncategorizedCount: Int = 0,
    val groups: List<TaskGroupStateModel> = emptyList(),
    val filter: TaskGroup? = null,
    val activity: List<ActivityEntryModel> = emptyList(),
    val lastScanAt: Instant? = null,
    val busy: String? = null,
    val aiBusy: String? = null,
    val stats: LibraryStatsModel = LibraryStatsModel(),
    val classifyCandidates: Int = 0,
    val tagCandidates: Int = 0,
    val inbox: InboxSummaryModel? = null,
    val upcoming: List<UpcomingRowModel> = emptyList(),
) {
    val openTaskCount: Int
        get() {
            return groups.sumOf { it.rows.size }
        }
}
