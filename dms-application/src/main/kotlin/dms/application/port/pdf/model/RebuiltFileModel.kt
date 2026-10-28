package dms.application.port.pdf.model

import dms.domain.document.model.ContentHashModel
import dms.domain.rebuild.model.RebuiltPageModel

/**
 * A rebuilt PDF waiting outside the library for the user's choice. [id] names it for the rebuilder only;
 * [size] is its size in bytes. [sourceHash] is the content of the scan it was made from, so a scan that
 * changed since is never replaced by it.
 */
data class RebuiltFileModel(
    val id: String,
    val pages: List<RebuiltPageModel>,
    val size: Long,
    val sourceHash: ContentHashModel,
)
