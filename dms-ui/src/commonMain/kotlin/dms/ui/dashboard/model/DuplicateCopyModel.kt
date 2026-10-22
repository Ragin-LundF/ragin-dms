package dms.ui.dashboard.model

import dms.ui.components.model.ActionButtonModel

/** One copy in a duplicates task: its [path], what is assigned to it, and the buttons to open or keep it. */
data class DuplicateCopyModel(
    val path: String,
    val detail: String,
    val open: ActionButtonModel,
    val keep: ActionButtonModel,
)
