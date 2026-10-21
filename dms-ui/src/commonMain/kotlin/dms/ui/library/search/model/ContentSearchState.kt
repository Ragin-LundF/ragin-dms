package dms.ui.library.search.model

import dms.domain.document.DocumentId

/**
 * Searching the text of the documents: whether it is [enabled], whether a search is [searching]
 * now, and the documents whose text [matches] the current query.
 */
data class ContentSearchState(
    val enabled: Boolean = true,
    val searching: Boolean = false,
    val matches: Set<DocumentId> = emptySet(),
)
