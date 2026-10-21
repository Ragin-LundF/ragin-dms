package dms.ui.dashboard.stats

import dms.domain.catalog.Catalog
import dms.domain.catalog.CatalogRevision
import dms.domain.catalog.LibraryId
import dms.domain.document.ContentHash
import dms.domain.document.Document
import dms.domain.document.DocumentId
import dms.domain.document.DocumentKind
import dms.domain.document.DocumentMetadata
import dms.domain.library.RelativePath
import dms.domain.taxonomy.Category
import dms.domain.taxonomy.CategoryId
import dms.domain.taxonomy.TagName
import dms.ui.dashboard.stats.model.KindCount
import java.time.Duration
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneOffset
import kotlin.test.Test
import kotlin.test.assertEquals

class LibraryStatisticsTest {

    private val now = Instant.parse("2026-06-15T12:00:00Z")
    private val invoices = Category(id = CategoryId(value = "invoices"), name = "Invoices")
    private val tax = TagName.of(raw = "tax")
    private val home = TagName.of(raw = "home")

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
                KindCount(kind = DocumentKind.PDF, count = 2),
                KindCount(kind = DocumentKind.IMAGE, count = 1),
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

    private fun sampleCatalog(): Catalog {
        val documents = listOf(
            document(id = "a", path = "letters/a.pdf", age = Duration.ofDays(5), size = 100).copy(
                categoryId = invoices.id,
                tags = setOf(tax),
                metadata = DocumentMetadata(
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
                metadata = DocumentMetadata(documentDate = LocalDate.parse("2012-05-05")),
            ),
        )
        return Catalog(
            libraryId = LibraryId(value = "library"),
            revision = CatalogRevision.INITIAL,
            documents = documents.associateBy { it.id },
            categories = mapOf(invoices.id to invoices),
            tags = setOf(tax, home),
        )
    }

    private fun document(id: String, path: String, age: Duration, size: Long): Document {
        return Document(
            id = DocumentId(value = id),
            path = RelativePath.of(raw = path),
            contentHash = ContentHash(value = id.repeat(n = 64)),
            size = size,
            addedAt = now.minus(age),
        )
    }
}
