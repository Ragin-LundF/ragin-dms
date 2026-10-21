package dms.ui.dashboard.model

import dms.domain.task.TaskKey
import dms.ui.components.model.ActionButton

/** One open task as the dashboard shows it, with exactly the actions its use cases support. */
data class TaskRowState(
    val key: TaskKey,
    val title: String,
    val detail: String,
    val buttons: List<ActionButton>,
)
