package dms.ui.dashboard.attention.model

import androidx.compose.ui.graphics.vector.ImageVector
import dms.ui.components.model.ActionButtonModel
import dms.ui.dashboard.model.DuplicateCopyModel
import dms.ui.dashboard.model.TaskFileModel

/**
 * One row of "Needs your attention": a thumbnail of [preview] (or [icon]), a title, a one-line
 * detail, the [buttons] that act (each already names why it cannot be used now) and the [opens]
 * that only look at a file, so they always work. A duplicates task lists its [copies].
 */
data class AttentionRowModel(
    val key: String,
    val severity: AttentionSeverity,
    val title: String,
    val detail: String,
    val icon: ImageVector,
    val preview: TaskFileModel? = null,
    val buttons: List<ActionButtonModel> = emptyList(),
    val opens: List<ActionButtonModel> = emptyList(),
    val copies: List<DuplicateCopyModel> = emptyList(),
)
