package dms.domain.query

import dms.domain.TestData
import dms.domain.TestData.applied
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.field.FieldPresets
import dms.domain.field.FieldValue
import dms.domain.field.model.CurrencyCodeModel
import dms.domain.library.model.RelativePathModel
import dms.domain.query.model.DateRangeModel
import dms.domain.query.model.DocumentQueryModel
import dms.domain.query.model.FieldConditionModel
import dms.domain.query.model.QueryContextModel
import dms.domain.query.model.TagFilterModel
import dms.domain.settings.model.CatalogSettingsModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import dms.domain.taxonomy.model.TagNameModel
import java.math.BigDecimal
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneOffset
import kotlin.test.Test
import kotlin.test.assertEquals

class DocumentQueryTest {

    private val insurance = CategoryModel(id = CategoryIdModel(value = "insurance"), name = "Insurance")
    private val car = CategoryModel(id = CategoryIdModel(value = "car"), name = "Car", parentId = insurance.id)
    private val tax = TagNameModel.of(raw = "Tax")
    private val paid = TagNameModel.of(raw = "Paid")
    private val euro = CurrencyCodeModel(value = "EUR")

    private val policy = TestData.document(
        id = "policy",
        path = "insurance/policy.pdf",
        categoryId = car.id,
        tags = setOf(tax),
    )
    private val invoice = TestData.document(id = "invoice", path = "letters/invoice.pdf", tags = setOf(tax, paid))
    private val photo = TestData.document(id = "photo", path = "photo.jpg")

    private val catalog = TestData.catalogWith()
        .addCategory(category = insurance).applied()
        .addCategory(category = car).applied()
        .changeSettings(settings = CatalogSettingsModel(fieldDefinitions = FieldPresets.ALL)).applied()
        .addDocument(document = policy).applied()
        .addDocument(document = invoice).applied()
        .addDocument(document = photo).applied()
        .updateMetadata(
            documentId = invoice.id,
            metadata = DocumentMetadataModel(
                title = "Garage bill",
                documentDate = LocalDate.of(2025, 6, 1),
                correspondent = "Garage",
            ),
        ).applied()
        .setNote(documentId = photo.id, note = "Scratch on the left door").applied()
        .setFields(
            documentId = invoice.id,
            fields = mapOf(
                FieldPresets.AMOUNT.key to FieldValue.Money(amount = BigDecimal("250"), currency = euro),
                FieldPresets.DUE_DATE.key to FieldValue.Date(date = LocalDate.of(2026, 3, 20)),
            ),
        ).applied()
        .setFields(
            documentId = policy.id,
            fields = mapOf(FieldPresets.AMOUNT.key to FieldValue.Money(amount = BigDecimal("80"), currency = euro)),
        ).applied()

    private val context = QueryContextModel(catalog = catalog, today = LocalDate.of(2026, 3, 1), zone = ZoneOffset.UTC)

    private fun ids(query: DocumentQueryModel): Set<String> {
        return query.filter(context = context).map { it.id.value }.toSet()
    }

    @Test
    fun `an empty query matches everything`() {
        assertEquals(expected = setOf("policy", "invoice", "photo"), actual = ids(query = DocumentQueryModel.ALL))
    }

    @Test
    fun `text looks at path, title, correspondent, tags and note`() {
        assertEquals(expected = setOf("invoice"), actual = ids(query = DocumentQueryModel(text = "garage BILL")))
        assertEquals(expected = setOf("photo"), actual = ids(query = DocumentQueryModel(text = "left door")))
        assertEquals(expected = setOf("policy"), actual = ids(query = DocumentQueryModel(text = "insurance/")))
        assertEquals(expected = setOf("invoice"), actual = ids(query = DocumentQueryModel(text = "paid")))
    }

    @Test
    fun `a category includes its subcategories`() {
        assertEquals(expected = setOf("policy"), actual = ids(query = DocumentQueryModel(category = insurance.id)))
        assertEquals(
            expected = setOf("invoice", "photo"),
            actual = ids(query = DocumentQueryModel(uncategorized = true)),
        )
    }

    @Test
    fun `tags can be required all, any or none`() {
        assertEquals(
            expected = setOf("invoice"),
            actual = ids(query = DocumentQueryModel(tags = TagFilterModel(all = setOf(tax, paid)))),
        )
        assertEquals(
            expected = setOf("policy", "invoice"),
            actual = ids(query = DocumentQueryModel(tags = TagFilterModel(any = setOf(tax)))),
        )
        assertEquals(
            expected = setOf("policy", "photo"),
            actual = ids(query = DocumentQueryModel(tags = TagFilterModel(none = setOf(paid)))),
        )
        assertEquals(expected = setOf("photo"), actual = ids(query = DocumentQueryModel(untagged = true)))
    }

    @Test
    fun `dates use the document date, otherwise the day it was added`() {
        assertEquals(
            expected = setOf("invoice"),
            actual = ids(query = DocumentQueryModel(dates = DateRangeModel.year(year = 2025))),
        )
        assertEquals(
            expected = setOf("policy", "photo"),
            actual = ids(query = DocumentQueryModel(dates = DateRangeModel.untilEndOf(year = 2024))),
        )
    }

    @Test
    fun `correspondent and folder narrow the result`() {
        assertEquals(expected = setOf("invoice"), actual = ids(query = DocumentQueryModel(correspondent = " garage ")))
        assertEquals(
            expected = setOf("policy"),
            actual = ids(query = DocumentQueryModel(folder = RelativePathModel.of(raw = "insurance"))),
        )
    }

    @Test
    fun `field conditions compare amounts and dates`() {
        val expensive = FieldConditionModel(
            key = FieldPresets.AMOUNT.key,
            comparison = FieldComparison.GREATER_THAN,
            value = FieldValue.Number(number = BigDecimal("100")),
        )
        val dueBefore = FieldConditionModel(
            key = FieldPresets.DUE_DATE.key,
            comparison = FieldComparison.LESS_THAN,
            value = FieldValue.Date(date = LocalDate.of(2026, 12, 1)),
        )
        val dueSoon = FieldConditionModel(
            key = FieldPresets.DUE_DATE.key,
            comparison = FieldComparison.WITHIN_DAYS,
            value = FieldValue.Number(number = BigDecimal("30")),
        )
        val noAmount = FieldConditionModel(key = FieldPresets.AMOUNT.key, comparison = FieldComparison.ABSENT)

        assertEquals(expected = setOf("invoice"), actual = ids(query = DocumentQueryModel(fields = listOf(expensive))))
        assertEquals(expected = setOf("invoice"), actual = ids(query = DocumentQueryModel(fields = listOf(dueBefore))))
        assertEquals(expected = setOf("invoice"), actual = ids(query = DocumentQueryModel(fields = listOf(dueSoon))))
        assertEquals(expected = setOf("photo"), actual = ids(query = DocumentQueryModel(fields = listOf(noAmount))))
    }

    @Test
    fun `values of different types never match`() {
        val wrongType = FieldConditionModel(
            key = FieldPresets.AMOUNT.key,
            comparison = FieldComparison.EQUALS,
            value = FieldValue.Date(date = LocalDate.of(2026, 1, 1)),
        )

        assertEquals(expected = emptySet(), actual = ids(query = DocumentQueryModel(fields = listOf(wrongType))))
    }

    @Test
    fun `added since uses the time a document was added`() {
        assertEquals(
            expected = emptySet(),
            actual = ids(query = DocumentQueryModel(addedSince = Instant.parse("2024-02-01T00:00:00Z"))),
        )
    }

    @Test
    fun `conditions combine and the content term is left to the content search`() {
        val query = DocumentQueryModel(
            tags = TagFilterModel(any = setOf(tax)),
            category = insurance.id,
            contentText = "anything",
        )

        assertEquals(expected = setOf("policy"), actual = ids(query = query))
        assertEquals(expected = true, actual = query.needsContent)
    }

    @Test
    fun `a query naming a category, tag or field the catalog lacks refers to something missing`() {
        val removed = CategoryIdModel(value = "removed")

        assertEquals(
            expected = false,
            actual = DocumentQueryModel(
                category = car.id,
                tags = TagFilterModel(any = setOf(tax)),
            ).refersToMissing(catalog),
        )
        assertEquals(
            expected = true,
            actual = DocumentQueryModel(category = removed).refersToMissing(catalog = catalog),
        )
        assertEquals(
            expected = true,
            actual = DocumentQueryModel(
                tags = TagFilterModel(none = setOf(TagNameModel.of(raw = "gone"))),
            ).refersToMissing(catalog),
        )
        val unknownField = FieldConditionModel(
            key = dms.domain.field.model.FieldKeyModel(value = "gone"),
            comparison = FieldComparison.PRESENT,
        )
        assertEquals(
            expected = true,
            actual = DocumentQueryModel(fields = listOf(unknownField)).refersToMissing(catalog = catalog),
        )
    }
}
