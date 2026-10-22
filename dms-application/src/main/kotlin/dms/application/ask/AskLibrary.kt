package dms.application.ask

import dms.application.ask.model.AskOutcome
import dms.application.ask.model.CitationModel
import dms.application.ask.model.ScoredChunkModel
import dms.application.common.model.LibrarySnapshotModel
import dms.application.library.CatalogStore
import dms.application.port.ai.EmbeddingClient
import dms.application.port.ai.model.EmbeddingResult
import dms.application.port.assistant.LibraryAssistant
import dms.application.port.assistant.model.AssistantAnswer
import dms.application.port.assistant.model.SourceChunkModel
import dms.application.search.SearchContents
import dms.domain.document.model.DocumentModel
import dms.domain.query.model.DocumentQueryModel
import java.util.Locale

/**
 * Answers a question about the library. The documents in [scope][invoke] are searched two ways: by
 * the words of the question in their text and metadata, and by meaning through the index. The best
 * passages go to the chat model, which answers with citations `[n]`; they are mapped back to the
 * documents. When nothing fits, the answer is an honest "not found" without asking the AI.
 */
class AskLibrary(
    private val store: CatalogStore,
    private val search: SearchContents,
    private val texts: DocumentTexts,
    private val embeddings: EmbeddingClient,
    private val vectors: VectorSearch,
    private val assistant: LibraryAssistant,
) {

    suspend operator fun invoke(question: String, scope: DocumentQueryModel = DocumentQueryModel.ALL): AskOutcome {
        val library = store.current() ?: return AskOutcome.LibraryNotOpen
        val documents = search.search(query = scope)
        val semantic = semanticChunks(library = library, documents = documents, question = question)
        val words = wordsOf(question = question)
        val lexical = keywordChunks(library = library, documents = documents, words = words)
        val best = merge(semantic = semantic.orEmpty(), lexical = lexical).take(n = SOURCES)
        if (best.isEmpty()) {
            return AskOutcome.NotFound
        }
        val sources = best.mapIndexed { index, chunk -> sourceOf(library = library, chunk = chunk, number = index + 1) }
        return when (val answer = assistant.answer(question = question, sources = sources)) {
            is AssistantAnswer.Failed -> AskOutcome.Failed(reason = answer.reason, detail = answer.detail)
            is AssistantAnswer.Tags -> AskOutcome.NotFound
            is AssistantAnswer.Text -> AskOutcome.Answered(
                text = answer.text,
                citations = citationsOf(answer = answer.text, chunks = best, library = library),
                model = answer.model,
                usedVectors = semantic != null,
            )
        }
    }

    /** The passages closest in meaning, or `null` when the index or the embedding model is not there. */
    private suspend fun semanticChunks(
        library: LibrarySnapshotModel,
        documents: List<DocumentModel>,
        question: String,
    ): List<ScoredChunkModel>? {
        val query = embeddings.embed(texts = listOf(question)) as? EmbeddingResult.Vectors ?: return null
        val vector = query.vectors.firstOrNull() ?: return null
        val scored = vectors.score(
            folder = library.folders.primary,
            documents = documents,
            query = vector,
            model = query.model,
        )
        return scored.takeIf { it.isNotEmpty() }?.filter { it.score >= MIN_SIMILARITY }
    }

    /** For each document with words of the question, its passage with the most of them. */
    private suspend fun keywordChunks(
        library: LibrarySnapshotModel,
        documents: List<DocumentModel>,
        words: Set<String>,
    ): List<ScoredChunkModel> {
        if (words.isEmpty()) {
            return emptyList()
        }
        return documents.mapNotNull { document ->
            val header = TextChunks.header(document = document, catalog = library.catalog).lowercase(Locale.ROOT)
            val text = texts.of(folder = library.folders.primary, document = document)
            TextChunks.of(text = text).map { chunk ->
                val haystack = header + "\n" + chunk.text.lowercase(Locale.ROOT)
                val hits = words.count { haystack.contains(other = it) }
                ScoredChunkModel(
                    documentId = document.id,
                    start = chunk.start,
                    end = chunk.end,
                    score = hits.toDouble() / words.size,
                )
            }.filter { it.score > 0.0 }.maxByOrNull { it.score }
        }
    }

    /** Both lists by score, a passage found both ways counting twice, one passage per place. */
    private fun merge(semantic: List<ScoredChunkModel>, lexical: List<ScoredChunkModel>): List<ScoredChunkModel> {
        val combined = (semantic + lexical.map { it.copy(score = it.score * KEYWORD_WEIGHT) })
            .groupBy { Triple(it.documentId, it.start, it.end) }
            .map { (_, found) -> found.first().copy(score = found.sumOf { it.score }) }
        return combined.sortedByDescending { it.score }
    }

    private suspend fun sourceOf(
        library: LibrarySnapshotModel,
        chunk: ScoredChunkModel,
        number: Int,
    ): SourceChunkModel {
        val document = library.catalog.documents.getValue(key = chunk.documentId)
        val text = texts.of(folder = library.folders.primary, document = document).trim()
        val passage = text.substring(
            startIndex = chunk.start.coerceAtMost(maximumValue = text.length),
            endIndex = chunk.end.coerceAtMost(maximumValue = text.length),
        )
        return SourceChunkModel(
            number = number,
            title = document.metadata.title ?: document.path.fileName,
            text = TextChunks.header(document = document, catalog = library.catalog) + "\n\n" + passage,
        )
    }

    private fun citationsOf(
        answer: String,
        chunks: List<ScoredChunkModel>,
        library: LibrarySnapshotModel,
    ): List<CitationModel> {
        return CITATION.findAll(input = answer)
            .mapNotNull { it.groupValues[1].toIntOrNull() }
            .distinct()
            .mapNotNull { number ->
                val chunk = chunks.getOrNull(index = number - 1) ?: return@mapNotNull null
                val document = library.catalog.documents[chunk.documentId] ?: return@mapNotNull null
                CitationModel(
                    number = number,
                    documentId = document.id,
                    title = document.metadata.title ?: document.path.fileName,
                )
            }
            .toList()
    }

    private fun wordsOf(question: String): Set<String> {
        return WORD.findAll(input = question.lowercase(Locale.ROOT))
            .map { it.value }
            .filter { it.length >= MIN_WORD }
            .toSet()
    }

    private companion object {
        const val SOURCES = 8
        const val MIN_SIMILARITY = 0.3
        const val KEYWORD_WEIGHT = 0.5
        const val MIN_WORD = 4
        val WORD = Regex(pattern = "[\\p{L}\\p{N}]+")
        val CITATION = Regex(pattern = "\\[(\\d+)]")
    }
}
