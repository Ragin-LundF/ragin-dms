package dms.ui.pdf.model

import androidx.compose.ui.graphics.ImageBitmap
import dms.domain.document.model.DocumentIdModel

/**
 * The open page editor of one PDF document: its pages in their new order, the page pictures as they
 * load, and whether splitting moves the original to the trash.
 */
data class PageEditorStateModel(
    val documentId: DocumentIdModel,
    val name: String,
    val pages: List<PageItemModel> = emptyList(),
    val thumbnails: Map<Int, ImageBitmap> = emptyMap(),
    val trashOriginal: Boolean = false,
    val loading: Boolean = true,
) {
    /** `true` if pages were turned, removed or moved, so the file gets a new version. */
    val edited: Boolean
        get() {
            return pages.any { it.rotation != 0 || it.deleted } || pages.map { it.source } != pages.indices.toList()
        }

    val splits: Int
        get() {
            return pages.dropLast(n = 1).count { it.splitAfter && !it.deleted }
        }
}
