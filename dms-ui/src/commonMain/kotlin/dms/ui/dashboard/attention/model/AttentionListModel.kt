package dms.ui.dashboard.attention.model

import dms.ui.components.model.ActionButtonModel

/**
 * What "Needs your attention" shows: the most urgent [rows], at most a handful, out of [total], and
 * the [headerButtons] that do not belong to one row.
 */
data class AttentionListModel(
    val rows: List<AttentionRowModel> = emptyList(),
    val total: Int = 0,
    val headerButtons: List<ActionButtonModel> = emptyList(),
)
