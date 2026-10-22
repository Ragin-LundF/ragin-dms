package dms.application.ask

import dms.application.ask.model.TextChunkModel
import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentModel

/**
 * Cuts a document's text into passages of about 800 tokens that overlap, at blanks where possible,
 * and names the document in a short header that goes in front of every passage, so a question about
 * a title, a sender or a date finds it too.
 */
object TextChunks {

    const val SIZE: Int = 3_200
    const val OVERLAP: Int = 400
    const val MAX_CHUNKS: Int = 60

    fun of(text: String): List<TextChunkModel> {
        val trimmed = text.trim()
        if (trimmed.isEmpty()) {
            return listOf(TextChunkModel(start = 0, end = 0, text = ""))
        }
        val chunks = mutableListOf<TextChunkModel>()
        var start = 0
        while (start < trimmed.length && chunks.size < MAX_CHUNKS) {
            val hardEnd = minOf(start + SIZE, trimmed.length)
            val blank = trimmed.lastIndexOf(char = ' ', startIndex = hardEnd).takeIf { it > start + SIZE / 2 }
            val end = if (hardEnd == trimmed.length) hardEnd else blank ?: hardEnd
            chunks += TextChunkModel(
                start = start,
                end = end,
                text = trimmed.substring(startIndex = start, endIndex = end),
            )
            if (end == trimmed.length) {
                break
            }
            start = maxOf(end - OVERLAP, start + 1)
        }
        return chunks
    }

    /** Title, correspondent, category, date, tags and note of [document], one per line. */
    fun header(document: DocumentModel, catalog: CatalogModel): String {
        val category = generateSequence(seed = document.categoryId?.let { catalog.categories[it] }) { current ->
            current.parentId?.let { catalog.categories[it] }
        }.take(n = catalog.categories.size).map { it.name }.toList().reversed().joinToString(separator = " / ")
        return listOfNotNull(
            "Document: ${document.metadata.title ?: document.path.fileName}",
            document.metadata.correspondent?.let { "From: $it" },
            category.takeIf { it.isNotEmpty() }?.let { "Category: $it" },
            document.metadata.documentDate?.let { "Date: $it" },
            document.tags.takeIf { it.isNotEmpty() }
                ?.let { tags -> "Tags: ${tags.sorted().joinToString { it.value }}" },
            document.note?.let { "Note: $it" },
        ).joinToString(separator = "\n")
    }
}
