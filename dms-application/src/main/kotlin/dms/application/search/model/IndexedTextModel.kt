package dms.application.search.model

import dms.domain.document.model.ContentHashModel

/** The lower-case searchable [text] of a document, as it was for the file with [contentHash]. */
data class IndexedTextModel(
    val contentHash: ContentHashModel,
    val text: String,
)
