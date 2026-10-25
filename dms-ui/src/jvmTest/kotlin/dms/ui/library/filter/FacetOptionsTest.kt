package dms.ui.library.filter

import dms.domain.document.DocumentKind
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.library.filter.model.Facet
import dms.ui.library.filter.model.FacetFilter
import dms.ui.library.filter.model.FacetOptionModel
import dms.ui.support.SyntheticCatalog
import java.time.LocalDate
import java.time.ZoneOffset
import kotlin.test.Test
import kotlin.test.assertEquals

class FacetOptionsTest {

    private val tax = TagNameModel.of(raw = "tax")
    private val invoice = TagNameModel.of(raw = "invoice")
    private val documents = listOf(
        SyntheticCatalog.document(
            id = "1",
            path = "Home/Energy/a.pdf",
            categoryId = SyntheticCatalog.energy.id,
            tags = setOf(tax, invoice),
            documentDate = LocalDate.parse("2025-03-01"),
        ),
        SyntheticCatalog.document(
            id = "2",
            path = "Home/b.png",
            categoryId = SyntheticCatalog.invoices.id,
            tags = setOf(invoice),
        ),
        SyntheticCatalog.document(id = "3", path = "c.pdf", tags = setOf(invoice)),
    )
    private val options = FacetOptions.of(
        documents = documents,
        catalog = SyntheticCatalog.catalog(documents = documents.toTypedArray()),
        zone = ZoneOffset.UTC,
    )

    @Test
    fun `categories keep their tree, count their subcategories and end with the documents without one`() {
        assertEquals(
            expected = listOf(
                FacetOptionModel(
                    filter = FacetFilter.Category(categoryId = SyntheticCatalog.invoices.id),
                    label = "Invoices",
                    count = 2,
                ),
                FacetOptionModel(
                    filter = FacetFilter.Category(categoryId = SyntheticCatalog.energy.id),
                    label = "Energy",
                    count = 1,
                    depth = 1,
                ),
                FacetOptionModel(
                    filter = FacetFilter.Category(categoryId = SyntheticCatalog.letters.id),
                    label = "Letters",
                    count = 0,
                ),
                FacetOptionModel(
                    filter = FacetFilter.Category(categoryId = null),
                    label = "Without a category",
                    count = 1,
                    missing = true,
                ),
            ),
            actual = options.getValue(key = Facet.CATEGORY),
        )
    }

    @Test
    fun `tags come by how often they are used, years newest first, and only the kinds the documents have`() {
        assertEquals(expected = listOf("invoice" to 3, "tax" to 1), actual = labelsOf(facet = Facet.TAG))
        assertEquals(expected = listOf("2025" to 1, "2024" to 2), actual = labelsOf(facet = Facet.YEAR))
        assertEquals(
            expected = listOf(FacetFilter.Type(kind = DocumentKind.PDF), FacetFilter.Type(kind = DocumentKind.IMAGE)),
            actual = options.getValue(key = Facet.TYPE).map { it.filter },
        )
    }

    @Test
    fun `folders form a tree with the documents below each`() {
        val folders = options.getValue(key = Facet.FOLDER)

        assertEquals(expected = listOf("Home" to 2, "Energy" to 1), actual = labelsOf(facet = Facet.FOLDER))
        assertEquals(expected = listOf(0, 1), actual = folders.map { it.depth })
    }

    private fun labelsOf(facet: Facet): List<Pair<String, Int>> {
        return options.getValue(key = facet).map { it.label to it.count }
    }
}
