package dms.application.ask

import dms.application.ask.model.RelatedOutcome
import dms.application.library.CatalogStore
import dms.domain.document.model.DocumentIdModel

/** The documents closest in meaning to one document, by the average of their vectors in the index. */
class RelatedDocuments(
    private val store: CatalogStore,
    private val vectors: VectorSearch,
) {

    suspend operator fun invoke(documentId: DocumentIdModel, limit: Int = DEFAULT_LIMIT): RelatedOutcome {
        val library = store.current() ?: return RelatedOutcome.LibraryNotOpen
        val document = library.catalog.documents[documentId] ?: return RelatedOutcome.NotFound
        val folder = library.folders.primary
        val (model, centroid) = vectors.centroid(folder = folder, document = document)
            ?: return RelatedOutcome.Found(documentIds = emptyList())
        val related = library.catalog.documents.values
            .filter { it.id != documentId }
            .mapNotNull { other ->
                val (otherModel, otherCentroid) = vectors.centroid(folder = folder, document = other)
                    ?: return@mapNotNull null
                val comparable = otherModel == model && otherCentroid.size == centroid.size
                if (comparable) other.id to VectorSearch.cosine(first = centroid, second = otherCentroid) else null
            }
            .filter { it.second >= MIN_SIMILARITY }
            .sortedByDescending { it.second }
            .take(n = limit)
            .map { it.first }
        return RelatedOutcome.Found(documentIds = related)
    }

    private companion object {
        const val DEFAULT_LIMIT = 5
        const val MIN_SIMILARITY = 0.5
    }
}
