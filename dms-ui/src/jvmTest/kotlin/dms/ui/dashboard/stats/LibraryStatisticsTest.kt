package dms.ui.dashboard.stats

import dms.domain.catalog.model.CatalogModel
import dms.domain.catalog.model.CatalogRevisionModel
import dms.domain.catalog.model.LibraryIdModel
import dms.domain.document.DocumentKind
import dms.domain.document.model.ContentHashModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.dashboard.stats.model.KindCountModel
import java.time.Duration
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneOffset
import kotlin.test.Test
import kotlin.test.assertEquals

class LibraryStatisticsTest {

    private val now = Instant.parse("2026-06-15T12:00:00Z")
    private val invoices = CategoryModel(id = CategoryIdModel(value = "invoices"), name = "Invoices")
    private val tax = TagNameModel.of(raw = "tax")
    private val home = TagNameModel.of(raw = "home")

    @Test
    fun `no catalog gives empty figures`() {
        val stats = LibraryStatistics.of(catalog = null, now = now, zone = ZoneOffset.UTC)

        assertEquals(expected = 0, actual = stats.documentCount)
        assertEquals(expected = 0f, actual = stats.categorizedShare)
        assertEquals(expected = emptyList(), actual = stats.byYear)
    }

    @Test
    fun `counts documents, sizes, recent additions and missing categories`() {
        val stats = LibraryStatistics.of(catalog = sampleCatalog(), now = now, zone = ZoneOffset.UTC)

        assertEquals(expected = 3, actual = stats.documentCount)
        assertEquals(expected = 350L, actual = stats.totalBytes)
        assertEquals(expected = 1, actual = stats.addedRecently)
        assertEquals(expected = 1, actual = stats.uncategorizedCount)
        assertEquals(
            expected = listOf(
                KindCountModel(kind = DocumentKind.PDF, count = 2),
                KindCountModel(kind = DocumentKind.IMAGE, count = 1),
            ),
            actual = stats.byKind,
        )
    }

    @Test
    fun `ranks categories, tags and correspondents by count`() {
        val stats = LibraryStatistics.of(catalog = sampleCatalog(), now = now, zone = ZoneOffset.UTC)

        assertEquals(expected = listOf("invoices" to 2), actual = stats.categories.map { it.id to it.value })
        assertEquals(expected = listOf("tax" to 2, "home" to 1), actual = stats.tags.map { it.label to it.value })
        assertEquals(expected = listOf("Sample Utility" to 1), actual = stats.correspondents.map { it.id to it.value })
    }

    @Test
    fun `years fold older documents into the first bar`() {
        val stats = LibraryStatistics.of(catalog = sampleCatalog(), now = now, zone = ZoneOffset.UTC)

        assertEquals(expected = 8, actual = stats.byYear.size)
        assertEquals(expected = "≤2019" to 1, actual = stats.byYear.first().let { it.label to it.value })
        assertEquals(expected = 1, actual = stats.byYear.single { it.label == "2025" }.value)
        assertEquals(expected = 1, actual = stats.byYear.single { it.label == "2026" }.value)
    }

    @Test
    fun `recent documents are newest first with title or file name`() {
        val stats = LibraryStatistics.of(catalog = sampleCatalog(), now = now, zone = ZoneOffset.UTC)

        assertEquals(expected = listOf("Sample bill", "b.pdf", "c.jpg"), actual = stats.recent.map { it.title })
        assertEquals(expected = "Invoices", actual = stats.recent.first().categoryName)
        assertEquals(expected = "letters", actual = stats.recent.first().folder)
    }

    private fun sampleCatalog(): CatalogModel {
        val documents = listOf(
            document(id = "a", path = "letters/a.pdf", age = Duration.ofDays(5), size = 100).copy(
                categoryId = invoices.id,
                tags = setOf(tax),
                metadata = DocumentMetadataModel(
                    title = "Sample bill",
                    documentDate = LocalDate.parse("2025-03-01"),
                    correspondent = "Sample Utility",
                ),
            ),
            document(id = "b", path = "letters/b.pdf", age = Duration.ofDays(60), size = 200).copy(
                categoryId = invoices.id,
                tags = setOf(tax, home),
            ),
            document(id = "c", path = "c.jpg", age = Duration.ofDays(5000), size = 50).copy(
                metadata = DocumentMetadataModel(documentDate = LocalDate.parse("2012-05-05")),
            ),
        )
        return CatalogModel(
            libraryId = LibraryIdModel(value = "library"),
            revision = CatalogRevisionModel.INITIAL,
            documents = documents.associateBy { it.id },
            categories = mapOf(invoices.id to invoices),
            tags = setOf(tax, home),
        )
    }

    private fun document(id: String, path: String, age: Duration, size: Long): DocumentModel {
        return DocumentModel(
            id = DocumentIdModel(value = id),
            path = RelativePathModel.of(raw = path),
            contentHash = ContentHashModel(value = id.repeat(n = 64)),
            size = size,
            addedAt = now.minus(age),
        )
    }
}
