package dms.application.pdf

import dms.application.library.CatalogStore
import dms.application.pdf.model.SearchableProgressModel
import dms.domain.document.DocumentKind
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow

/**
 * Makes every PDF of the library searchable that has no text layer yet, one after the other; PDFs
 * with text are skipped. Cancelling the collection stops it between two documents; pausing is
 * stopping and starting again, since done documents are skipped.
 */
class MakeAllSearchable(
    private val store: CatalogStore,
    private val makeSearchable: MakeSearchable,
) {

    operator fun invoke(precise: Boolean = false): Flow<SearchableProgressModel> {
        return flow {
            val library = store.current() ?: return@flow
            val pdfs = library.catalog.documents.values
                .filter { it.kind == DocumentKind.PDF }
                .sortedBy { it.path.value }
            pdfs.forEachIndexed { index, document ->
                val outcome = makeSearchable(documentId = document.id, precise = precise)
                emit(
                    value = SearchableProgressModel(
                        done = index + 1,
                        total = pdfs.size,
                        documentId = document.id,
                        outcome = outcome,
                    ),
                )
            }
        }
    }
}
