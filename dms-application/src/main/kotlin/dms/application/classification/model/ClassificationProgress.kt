package dms.application.classification.model

import dms.domain.document.DocumentId

/** Progress of a batch classification: [done] of [total] documents, with the outcome of the last one. */
data class ClassificationProgress(
    val done: Int,
    val total: Int,
    val documentId: DocumentId,
    val outcome: SuggestOutcome,
)
