package dms.ui.library

import androidx.compose.ui.unit.Dp
import dms.ui.theme.Spacing

/** The column widths of the document list, shared by its header and its rows so they line up. */
object DocumentColumns {
    const val NAME: Float = 3f
    const val CATEGORY: Float = 1.3f
    const val TAGS: Float = 1.6f
    const val CORRESPONDENT: Float = 1.4f
    const val DATE: Float = 0.9f
    const val SIZE: Float = 0.7f
    val padding: Dp = Spacing.medium
    val rowPadding: Dp = Spacing.small
    val gap: Dp = Spacing.medium
    val chipGap: Dp = Spacing.xsmall
    val thumbnailWidth: Dp = Dp(value = 22f)
    val thumbnailHeight: Dp = Dp(value = 28f)
}
