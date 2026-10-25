package dms.application.search.model

import dms.domain.document.model.DocumentIdModel

sealed interface SearchOutcome {

    /** The documents whose text contains the query; none for a blank query. */
    data class Found(val documentIds: Set<DocumentIdModel>) : SearchOutcome

    data object LibraryNotOpen : SearchOutcome
}
