package dms.application.library.model

import dms.application.port.state.model.ActivityEntry
import dms.domain.task.OpenTask

/** Everything the dashboard shows. */
data class Dashboard(
    val tasks: List<OpenTask>,
    val folders: List<FolderState>,
    val documentCount: Int,
    val uncategorizedCount: Int,
    val recentActivity: List<ActivityEntry>,
)
