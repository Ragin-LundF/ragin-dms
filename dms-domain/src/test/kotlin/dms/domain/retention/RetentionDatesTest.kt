package dms.domain.retention

import dms.domain.TestData
import dms.domain.TestData.applied
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.retention.model.RetentionModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import java.time.Instant
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class RetentionDatesTest {

    private val tax = CategoryModel(id = CategoryIdModel(value = "tax"), name = "Tax")
    private val returns = CategoryModel(id = CategoryIdModel(value = "returns"), name = "Returns", parentId = tax.id)
    private val document = TestData.document(id = "return", path = "return.pdf", categoryId = returns.id)
    private val catalog = TestData.catalogWith()
        .addCategory(category = tax).applied()
        .addCategory(category = returns).applied()
        .addDocument(document = document).applied()
        .updateMetadata(
            documentId = document.id,
            metadata = DocumentMetadataModel(documentDate = LocalDate.of(2020, 4, 15)),
        )
        .applied()

    private fun dueDate(catalog: dms.domain.catalog.model.CatalogModel): LocalDate? {
        return RetentionDates.dueDate(document = catalog.documents.getValue(document.id), catalog = catalog)
    }

    @Test
    fun `nothing has a retention unless the user sets one`() {
        assertNull(actual = dueDate(catalog = catalog))
    }

    @Test
    fun `a category period counts from the end of the document date's year, inherited by subcategories`() {
        val withPeriod = catalog.setRetentionYears(categoryId = tax.id, years = 10).applied()

        assertEquals(expected = LocalDate.of(2030, 12, 31), actual = dueDate(catalog = withPeriod))
    }

    @Test
    fun `the document's own keep-until date wins over the category`() {
        val withBoth = catalog.setRetentionYears(categoryId = tax.id, years = 10).applied()
            .setRetention(
                documentId = document.id,
                retention = RetentionModel(keepUntil = LocalDate.of(2027, 1, 31)),
            ).applied()

        assertEquals(expected = LocalDate.of(2027, 1, 31), actual = dueDate(catalog = withBoth))
    }

    @Test
    fun `a settled retention is never flagged again, also not through the category`() {
        val settled = catalog.setRetentionYears(categoryId = tax.id, years = 1).applied()
            .setRetention(
                documentId = document.id,
                retention = RetentionModel(settledAt = Instant.parse("2026-01-01T00:00:00Z")),
            )
            .applied()

        assertNull(actual = dueDate(catalog = settled))
    }
}
