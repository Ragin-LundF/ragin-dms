package dms.application.library.model

import dms.domain.document.model.ContentHashModel
import dms.domain.library.model.RelativePathModel

/** A file the scan of the primary folder read and the catalog did not know at the start of the scan. */
data class ScannedFileModel(
    val path: RelativePathModel,
    val size: Long,
    val contentHash: ContentHashModel,
)
