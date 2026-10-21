package dms.ui.dashboard.model

import dms.ui.components.model.ActionButton

/** One copy in a duplicates task: its [path], what is assigned to it, and the buttons to open or keep it. */
data class DuplicateCopy(
    val path: String,
    val detail: String,
    val open: ActionButton,
    val keep: ActionButton,
)
