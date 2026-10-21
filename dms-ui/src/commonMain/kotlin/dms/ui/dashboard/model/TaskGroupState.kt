package dms.ui.dashboard.model

import dms.ui.components.DialogController
import dms.ui.components.model.ActionButton

/** The open tasks of one kind; [bulkAction] applies the default action to all of them. */
data class TaskGroupState(
    val group: TaskGroup,
    val rows: List<TaskRowState>,
    val bulkAction: ActionButton? = null,
)
