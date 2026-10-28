package dms.ui.pdf.model

import androidx.compose.ui.graphics.ImageBitmap
import dms.application.port.pdf.model.RebuiltFileModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.RelativePathModel

/**
 * The comparison of a scan with its rebuild, open until the user chooses which to keep. [page] counts
 * from 0; [original] and [rebuiltImage] are its pictures once loaded. [overlay] marks what became text,
 * what stayed a line image and what stayed a picture.
 */
data class RebuildCompareStateModel(
    val documentId: DocumentIdModel,
    val path: RelativePathModel,
    val rebuilt: RebuiltFileModel,
    val originalSize: Long,
    val page: Int = 0,
    val view: CompareView = CompareView.SIDE_BY_SIDE,
    val overlay: Boolean = false,
    val original: ImageBitmap? = null,
    val rebuiltImage: ImageBitmap? = null,
) {
    val pages: Int
        get() {
            return rebuilt.pages.size
        }
}
