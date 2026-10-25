package dms.application.ask.model

import dms.domain.document.model.DocumentIdModel

sealed interface RelatedOutcome {

    /** The closest documents, most similar first; none while the document is not in the index. */
    data class Found(val documentIds: List<DocumentIdModel>) : RelatedOutcome

    data object LibraryNotOpen : RelatedOutcome

    data object NotFound : RelatedOutcome
}
