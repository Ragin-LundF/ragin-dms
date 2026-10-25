package dms.ui.document.model

import dms.domain.document.model.DocumentIdModel

/**
 * The quick look: the [shown] document, or `null` while it is closed, the documents in view [order]
 * that next and previous move through, and the fact the user is [editing] in place.
 */
data class QuickLookStateModel(
    val shown: DocumentIdModel? = null,
    val order: List<DocumentIdModel> = emptyList(),
    val editing: QuickFact? = null,
)
