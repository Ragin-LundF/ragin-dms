package dms.domain.change.model

import dms.domain.document.model.ContentHashModel

/**
 * What a scan found out about one file.
 *
 * [contentHash] is `null` when the scanner skipped hashing because the file is unchanged since its
 * last scan, so that a scan does not download files a sync client keeps only remotely. Files the
 * catalog does not know are always hashed.
 */
data class FileFactsModel(
    val size: Long,
    val contentHash: ContentHashModel?,
) {
    init {
        require(value = size >= 0) { "Size must not be negative" }
    }
}
