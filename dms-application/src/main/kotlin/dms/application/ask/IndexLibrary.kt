package dms.application.ask

import dms.application.ask.model.IndexProgressModel
import dms.application.library.CatalogStore
import dms.application.port.ai.EmbeddingClient
import dms.application.port.ai.model.EmbeddingResult
import dms.application.port.index.VectorIndex
import dms.application.port.index.model.DocumentVectorsModel
import dms.application.port.index.model.IndexedChunkModel
import dms.application.port.storage.model.FolderOperation
import dms.application.sync.Replicator
import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentModel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow

/**
 * Builds the embedding index of the library in the background, document by document: only what is
 * new or changed (other file, other metadata, other embedding model) is embedded again. Each entry
 * goes to `.dms-index/` of the primary and is replicated. Cancelling the collection stops it between
 * two documents; running it again goes on where it stopped.
 */
class IndexLibrary(
    private val store: CatalogStore,
    private val texts: DocumentTexts,
    private val embeddings: EmbeddingClient,
    private val index: VectorIndex,
    private val replicator: Replicator,
) {

    operator fun invoke(): Flow<IndexProgressModel> {
        return flow {
            val library = store.current() ?: return@flow
            val model = embeddings.model()
            val documents = library.catalog.documents.values.sortedBy { it.path.value }
            documents.forEachIndexed { position, document ->
                val header = TextChunks.header(document = document, catalog = library.catalog)
                val existing = index.read(folder = library.folders.primary, documentId = document.id)
                val metadataHash = hashOf(text = header)
                val current = model != null && existing != null &&
                    existing.isCurrent(model = model, contentHash = document.contentHash, metadataHash = metadataHash)
                val failure = if (current) null else build(
                    catalog = library.catalog,
                    document = document,
                    header = header,
                )
                emit(
                    value = IndexProgressModel(
                        done = position + 1,
                        total = documents.size,
                        documentId = document.id,
                        indexed = !current && failure == null,
                        failure = failure,
                    ),
                )
                if (failure != null) {
                    return@flow
                }
            }
        }
    }

    /** Embeds and stores one document; returns why it failed, or `null`. */
    private suspend fun build(catalog: CatalogModel, document: DocumentModel, header: String): String? {
        val library = store.current() ?: return null
        val text = texts.of(folder = library.folders.primary, document = document)
        val chunks = TextChunks.of(text = text)
        val vectors = mutableListOf<FloatArray>()
        var model: String? = null
        chunks.chunked(size = BATCH).forEach { batch ->
            when (val result = embeddings.embed(texts = batch.map { "$header\n\n${it.text}" })) {
                is EmbeddingResult.Failed -> return result.detail ?: result.reason.name
                is EmbeddingResult.Vectors -> {
                    vectors += result.vectors
                    model = result.model
                }
            }
        }
        val dimensions = vectors.firstOrNull()?.size ?: return "The AI server returned no vectors"
        val entry = DocumentVectorsModel(
            documentId = document.id,
            model = checkNotNull(value = model),
            dimensions = dimensions,
            contentHash = document.contentHash,
            metadataHash = hashOf(text = TextChunks.header(document = document, catalog = catalog)),
            chunks = chunks.zip(other = vectors) { chunk, vector ->
                IndexedChunkModel(start = chunk.start, end = chunk.end, vector = vector)
            },
        )
        replicator.replicate(
            folders = library.folders,
            onPrimary = { primary -> index.write(folder = primary, vectors = entry) },
            followUp = FolderOperation.CopyIndex(documentId = document.id),
        )
        return null
    }

    companion object {
        private const val BATCH = 16

        /** A short fingerprint of the header; a change of title, sender, category or tags rebuilds the entry. */
        fun hashOf(text: String): String {
            return Integer.toHexString(text.hashCode())
        }
    }
}
