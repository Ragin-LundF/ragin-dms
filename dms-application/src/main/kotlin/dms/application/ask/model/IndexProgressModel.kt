package dms.application.ask.model

import dms.domain.document.model.DocumentIdModel

/**
 * Progress of building the index: [done] of [total] documents; the last one was [indexed] anew or
 * was current already. [failure] names why the AI server could not make embeddings, which stops the run.
 */
data class IndexProgressModel(
    val done: Int,
    val total: Int,
    val documentId: DocumentIdModel,
    val indexed: Boolean,
    val failure: String? = null,
)
