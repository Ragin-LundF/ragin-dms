package dms.ui.pdf.model

/**
 * One page in the page editor: page [source] of the file (from 0), turned by [rotation] degrees,
 * [deleted] to leave it out, and [splitAfter] to start a new document after it.
 */
data class PageItemModel(
    val source: Int,
    val rotation: Int = 0,
    val deleted: Boolean = false,
    val splitAfter: Boolean = false,
)
