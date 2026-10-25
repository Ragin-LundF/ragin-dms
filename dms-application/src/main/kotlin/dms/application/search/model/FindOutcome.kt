package dms.application.search.model

import dms.domain.document.model.DocumentModel

sealed interface FindOutcome {

    /** The documents the query matches, by path. */
    data class Found(val documents: List<DocumentModel>) : FindOutcome

    data object LibraryNotOpen : FindOutcome
}
