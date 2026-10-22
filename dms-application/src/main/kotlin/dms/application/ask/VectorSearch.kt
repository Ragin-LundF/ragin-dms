package dms.application.ask

import dms.application.ask.model.ScoredChunkModel
import dms.application.port.index.VectorIndex
import dms.application.port.index.model.DocumentVectorsModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.LibraryFolderModel
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlin.math.sqrt

/**
 * Brute-force cosine search over the index, in memory: the vectors of a document are read once per
 * file version and kept. Fast enough for a personal library of some thousand documents.
 */
class VectorSearch(private val index: VectorIndex) {

    private val mutex = Mutex()
    private val cache = HashMap<String, DocumentVectorsModel>()

    /** Every passage of [documents] scored against [query]; documents without a current entry are left out. */
    suspend fun score(
        folder: LibraryFolderModel,
        documents: List<DocumentModel>,
        query: FloatArray,
        model: String,
    ): List<ScoredChunkModel> {
        return documents.flatMap { document ->
            val vectors = vectorsOf(folder = folder, document = document) ?: return@flatMap emptyList()
            if (vectors.model != model || vectors.dimensions != query.size) {
                return@flatMap emptyList()
            }
            vectors.chunks.map { chunk ->
                ScoredChunkModel(
                    documentId = document.id,
                    start = chunk.start,
                    end = chunk.end,
                    score = cosine(first = query, second = chunk.vector),
                )
            }
        }
    }

    /** The average of a document's vectors, for "related documents"; `null` without a current entry. */
    suspend fun centroid(folder: LibraryFolderModel, document: DocumentModel): Pair<String, FloatArray>? {
        val vectors = vectorsOf(folder = folder, document = document)?.takeIf { it.chunks.isNotEmpty() } ?: return null
        val sum = FloatArray(size = vectors.dimensions)
        vectors.chunks.forEach { chunk -> chunk.vector.forEachIndexed { i, value -> sum[i] += value } }
        return vectors.model to FloatArray(size = sum.size) { sum[it] / vectors.chunks.size }
    }

    private suspend fun vectorsOf(folder: LibraryFolderModel, document: DocumentModel): DocumentVectorsModel? {
        return mutex.withLock {
            val key = "${document.id}:${document.contentHash}"
            cache[key] ?: index.read(folder = folder, documentId = document.id)
                ?.takeIf { it.contentHash == document.contentHash }
                ?.also { cache[key] = it }
        }
    }

    companion object {

        fun cosine(first: FloatArray, second: FloatArray): Double {
            var dot = 0.0
            var firstNorm = 0.0
            var secondNorm = 0.0
            first.indices.forEach { i ->
                dot += first[i] * second[i]
                firstNorm += first[i] * first[i]
                secondNorm += second[i] * second[i]
            }
            if (firstNorm == 0.0 || secondNorm == 0.0) {
                return 0.0
            }
            return dot / (sqrt(firstNorm) * sqrt(secondNorm))
        }
    }
}
