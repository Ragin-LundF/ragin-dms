package dms.application.ask.model

import dms.domain.document.model.DocumentIdModel

/** A source the answer cites as `[number]`: the document it came from. */
data class CitationModel(
    val number: Int,
    val documentId: DocumentIdModel,
    val title: String,
)
