package dms.application.library.model

import dms.domain.library.RelativePath

/**
 * Where a scan of the library stands: folder [folderNumber] of [folderCount] is being read, [done] of
 * its [total] files are finished. [total] is `null` while the folder is still being listed. [current]
 * is the last finished file, and [hashed] tells whether its content had to be read.
 */
data class ScanProgress(
    val folderName: String,
    val folderNumber: Int,
    val folderCount: Int,
    val done: Int = 0,
    val total: Int? = null,
    val current: RelativePath? = null,
    val hashed: Boolean = false,
)
