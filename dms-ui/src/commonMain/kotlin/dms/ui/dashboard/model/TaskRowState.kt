package dms.ui.dashboard.model

import dms.domain.task.TaskKey
import dms.ui.components.model.ActionButton

/**
 * One open task as the dashboard shows it, with exactly the actions its use cases support, and
 * [opens] to look at the file it is about: one button per version when the folders disagree. The row
 * shows a thumbnail of [preview]. A duplicates task lists its [copies], each with its own buttons.
 */
data class TaskRowState(
    val key: TaskKey,
    val title: String,
    val detail: String,
    val buttons: List<ActionButton>,
    val opens: List<ActionButton>,
    val preview: TaskFile?,
    val copies: List<DuplicateCopy> = emptyList(),
)
