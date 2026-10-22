package dms.application.library.model

import dms.application.port.state.model.ActivityEntryModel
import dms.domain.deadline.model.UpcomingDateModel
import dms.domain.task.OpenTask

/** Everything the dashboard shows; [upcoming] are the next deadline and retention dates. */
data class DashboardModel(
    val tasks: List<OpenTask>,
    val folders: List<FolderStateModel>,
    val documentCount: Int,
    val uncategorizedCount: Int,
    val recentActivity: List<ActivityEntryModel>,
    val upcoming: List<UpcomingDateModel> = emptyList(),
)
