package dms.ui.theme

import androidx.compose.ui.unit.Dp

/**
 * The fixed sizes of the library's own components, quick look and the folder view included, from the
 * approved concept. Spacing between them comes from [Spacing]; these are the sizes of things.
 */
object LibrarySizes {
    /** The library sidebar folded into its icon rail. */
    val rail: Dp = Dp(value = 60f)
    val searchInputMinWidth: Dp = Dp(value = 180f)
    val popoverWidth: Dp = Dp(value = 260f)
    val cardMinWidth: Dp = Dp(value = 132f)
    val tileWidth: Dp = Dp(value = 148f)
    val tilePageWidth: Dp = Dp(value = 44f)
    val tilePageHeight: Dp = Dp(value = 56f)
    val tileStackHeight: Dp = Dp(value = 62f)
    val tilePageOffset: Dp = Dp(value = 16f)
    val selectionCircle: Dp = Dp(value = 20f)
    val circleBorder: Dp = Dp(value = 1.5f)
    val labelIcon: Dp = Dp(value = 14f)
    val quickLookWidth: Dp = Dp(value = 380f)
    val quickLookPreviewWidth: Dp = Dp(value = 195f)
    val quickLookPreviewHeight: Dp = Dp(value = 260f)
    val folderTileMinWidth: Dp = Dp(value = 150f)
    val folderTilesMaxHeight: Dp = Dp(value = 220f)
    val folderTreeIndent: Dp = Dp(value = 14f)
}
