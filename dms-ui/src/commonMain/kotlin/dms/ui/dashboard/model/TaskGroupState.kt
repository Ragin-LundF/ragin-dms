package dms.ui.dashboard.model

import dms.ui.components.model.ActionButton

/** The open tasks of one kind; [bulkActions] answer all of them at once. */
data class TaskGroupState(
    val group: TaskGroup,
    val rows: List<TaskRowState>,
    val bulkActions: List<ActionButton> = emptyList(),
)
