package dms.application.search

import dms.application.library.CatalogStore
import dms.application.search.model.SearchOutcome

/** Finds the documents whose text contains a query, ignoring case; see [ContentMatcher]. Never writes. */
class SearchContents(
    private val store: CatalogStore,
    private val matcher: ContentMatcher,
) {

    suspend operator fun invoke(query: String): SearchOutcome {
        val library = store.current() ?: return SearchOutcome.LibraryNotOpen
        return SearchOutcome.Found(documentIds = matcher.containing(library = library, wanted = query))
    }
}
