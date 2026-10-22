package dms.ui.dashboard.model

import dms.domain.task.model.TaskKeyModel
import dms.ui.components.model.ActionButtonModel

/**
 * One open task as the dashboard shows it, with exactly the actions its use cases support, and
 * [opens] to look at the file it is about: one button per version when the folders disagree. The row
 * shows a thumbnail of [preview]. A duplicates task lists its [copies], each with its own buttons.
 */
data class TaskRowStateModel(
    val key: TaskKeyModel,
    val title: String,
    val detail: String,
    val buttons: List<ActionButtonModel>,
    val opens: List<ActionButtonModel>,
    val preview: TaskFileModel?,
    val copies: List<DuplicateCopyModel> = emptyList(),
)
