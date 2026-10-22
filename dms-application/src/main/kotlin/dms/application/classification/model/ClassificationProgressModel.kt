package dms.application.classification.model

import dms.domain.document.model.DocumentIdModel

/** Progress of a batch classification: [done] of [total] documents, with the outcome of the last one. */
data class ClassificationProgressModel(
    val done: Int,
    val total: Int,
    val documentId: DocumentIdModel,
    val outcome: SuggestOutcome,
)
