package dms.application.search.model

import dms.domain.document.ContentHash

/** The lower-case searchable [text] of a document, as it was for the file with [contentHash]. */
data class IndexedText(
    val contentHash: ContentHash,
    val text: String,
)
