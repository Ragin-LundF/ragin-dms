package dms.ui.dashboard.model

import dms.ui.components.model.ActionButtonModel

/** The open tasks of one kind; [bulkActions] answer all of them at once. */
data class TaskGroupStateModel(
    val group: TaskGroup,
    val rows: List<TaskRowStateModel>,
    val bulkActions: List<ActionButtonModel> = emptyList(),
)
