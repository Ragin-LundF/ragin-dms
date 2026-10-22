package dms.ui.library.search.model

import dms.domain.document.model.DocumentIdModel

/**
 * Searching the text of the documents: whether it is [enabled], whether a search is [searching]
 * now, and the documents whose text [matches] the current query.
 */
data class ContentSearchStateModel(
    val enabled: Boolean = true,
    val searching: Boolean = false,
    val matches: Set<DocumentIdModel> = emptySet(),
)
