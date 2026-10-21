package dms.application.library.model

import dms.domain.document.ContentHash
import dms.domain.library.RelativePath

/** A file the scan of the primary folder read and the catalog did not know at the start of the scan. */
data class ScannedFile(
    val path: RelativePath,
    val size: Long,
    val contentHash: ContentHash,
)
