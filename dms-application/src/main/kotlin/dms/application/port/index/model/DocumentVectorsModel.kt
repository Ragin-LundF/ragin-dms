package dms.application.port.index.model

import dms.domain.document.model.ContentHashModel
import dms.domain.document.model.DocumentIdModel

/**
 * The embeddings of one document, made by [model] with [dimensions] values per vector, for the file
 * with [contentHash] and the metadata with [metadataHash]. A stale hash or another model means the
 * entry is rebuilt; it is derived data and never merged.
 */
class DocumentVectorsModel(
    val documentId: DocumentIdModel,
    val model: String,
    val dimensions: Int,
    val contentHash: ContentHashModel,
    val metadataHash: String,
    val chunks: List<IndexedChunkModel>,
) {
    init {
        require(value = chunks.all { it.vector.size == dimensions }) { "Every vector needs $dimensions values" }
    }

    fun isCurrent(model: String, contentHash: ContentHashModel, metadataHash: String): Boolean {
        return this.model == model && this.contentHash == contentHash && this.metadataHash == metadataHash
    }
}
