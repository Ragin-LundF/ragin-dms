package dms.application.pdf.model

import dms.domain.document.model.DocumentIdModel

/** Progress of making many scans searchable: [done] of [total], with the outcome of the last one. */
data class SearchableProgressModel(
    val done: Int,
    val total: Int,
    val documentId: DocumentIdModel,
    val outcome: SearchableOutcome,
)
