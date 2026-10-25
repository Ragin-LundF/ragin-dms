package dms.application.search

import dms.application.library.CatalogStore
import dms.application.search.model.FindOutcome
import dms.domain.query.model.DocumentQueryModel

/** Finds the documents a query matches, its content term included; see [ContentMatcher]. Never writes. */
class FindDocuments(
    private val store: CatalogStore,
    private val matcher: ContentMatcher,
) {

    suspend operator fun invoke(query: DocumentQueryModel): FindOutcome {
        val library = store.current() ?: return FindOutcome.LibraryNotOpen
        return FindOutcome.Found(documents = matcher.matching(library = library, query = query))
    }
}
