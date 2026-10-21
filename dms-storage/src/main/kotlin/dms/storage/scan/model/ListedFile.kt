package dms.storage.scan.model

import dms.domain.library.RelativePath
import java.nio.file.Path

/** A regular file found while listing a folder, before its content is looked at. */
data class ListedFile(
    val path: RelativePath,
    val entry: Path,
    val size: Long,
    val modified: Long,
)
