package dms.ui.library.documents

import dms.domain.document.model.DocumentModel
import dms.ui.library.documents.model.GroupKey
import dms.ui.library.model.DocumentRowModel
import dms.ui.support.SyntheticCatalog
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneOffset
import kotlin.test.Test
import kotlin.test.assertEquals

class DocumentGroupsTest {

    private val documents = listOf(
        SyntheticCatalog.document(
            id = "1",
            path = "a.pdf",
            documentDate = LocalDate.parse("2025-09-30"),
            correspondent = "Sample Utility",
            categoryId = SyntheticCatalog.energy.id,
        ),
        SyntheticCatalog.document(id = "2", path = "b.pdf", addedAt = Instant.parse("2025-10-02T08:00:00Z")),
        SyntheticCatalog.document(
            id = "3",
            path = "c.pdf",
            documentDate = LocalDate.parse("2025-10-20"),
            correspondent = "sample utility",
            categoryId = SyntheticCatalog.letters.id,
        ),
        SyntheticCatalog.document(
            id = "4",
            path = "d.pdf",
            documentDate = LocalDate.parse("2019-04-01"),
            correspondent = "Another Sender",
            categoryId = SyntheticCatalog.invoices.id,
        ),
    )
    private val catalog = SyntheticCatalog.catalog(documents = documents.toTypedArray())

    @Test
    fun `by month uses the document date or the day it was added, newest month first`() {
        assertEquals(
            expected = listOf(
                "October 2025" to listOf("2", "3"),
                "September 2025" to listOf("1"),
                "April 2019" to listOf("4"),
            ),
            actual = groupsBy(key = GroupKey.MONTH),
        )
    }

    @Test
    fun `by category groups under the top-level category, the documents without one last`() {
        assertEquals(
            expected = listOf(
                "Invoices" to listOf("1", "4"),
                "Letters" to listOf("3"),
                "Without a category" to listOf("2"),
            ),
            actual = groupsBy(key = GroupKey.CATEGORY),
        )
    }

    @Test
    fun `by sender ignores the case of the name, and unknown senders come last`() {
        assertEquals(
            expected = listOf(
                "Another Sender" to listOf("4"),
                "Sample Utility" to listOf("1", "3"),
                "Unknown sender" to listOf("2"),
            ),
            actual = groupsBy(key = GroupKey.SENDER),
        )
    }

    private fun groupsBy(key: GroupKey): List<Pair<String, List<String>>> {
        return DocumentGroups.of(rows = documents.map(::rowOf), key = key, zone = ZoneOffset.UTC, catalog = catalog)
            .map { group -> group.title to group.rows.map { it.document.id.value } }
    }

    private fun rowOf(document: DocumentModel): DocumentRowModel {
        return DocumentRowModel(document = document, categoryName = null)
    }
}
