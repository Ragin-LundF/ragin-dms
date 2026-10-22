package dms.storage.scan.model

import dms.domain.library.model.RelativePathModel
import java.nio.file.Path

/** A regular file found while listing a folder, before its content is looked at. */
data class ListedFileModel(
    val path: RelativePathModel,
    val entry: Path,
    val size: Long,
    val modified: Long,
)
