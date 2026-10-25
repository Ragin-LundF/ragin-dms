package dms.ui.library.filter

import dms.domain.document.DocumentKind
import dms.domain.library.model.RelativePathModel
import dms.domain.query.model.DateRangeModel
import dms.domain.query.model.DocumentQueryModel
import dms.domain.query.model.TagFilterModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.library.filter.model.FacetFilter
import dms.ui.library.filter.model.FilterTokenModel
import dms.ui.library.model.YearFilterModel
import dms.ui.support.SyntheticCatalog
import kotlin.test.Test
import kotlin.test.assertEquals

class FilterTokensTest {

    private val invoice = TagNameModel.of(raw = "invoice")
    private val catalog = SyntheticCatalog.catalog(
        SyntheticCatalog.document(id = "1", path = "a.pdf", tags = setOf(invoice), correspondent = "Sample Utility"),
        SyntheticCatalog.document(id = "2", path = "b.pdf", correspondent = "sample utility"),
        extraTags = setOf(TagNameModel.of(raw = "insurance")),
    )

    @Test
    fun `a filter replaces the filter of its own facet and goes to the end`() {
        val tag = FacetFilter.Tag(tag = invoice)
        val year = FacetFilter.Year(year = YearFilterModel(year = 2024))
        val otherYear = FacetFilter.Year(year = YearFilterModel(year = 2025))

        val filters = FilterTokens.with(
            filters = FilterTokens.with(filters = listOf(year), filter = tag),
            filter = otherYear,
        )

        assertEquals(expected = listOf(tag, otherYear), actual = filters)
    }

    @Test
    fun `each filter is a removable token of its facet's key and value`() {
        val tokens = listOf(
            FacetFilter.Category(categoryId = SyntheticCatalog.energy.id),
            FacetFilter.Category(categoryId = null),
            FacetFilter.Tag(tag = invoice),
            FacetFilter.Year(year = YearFilterModel(year = 2025)),
            FacetFilter.Correspondent(name = "Sample Utility"),
            FacetFilter.Type(kind = DocumentKind.PDF),
            FacetFilter.Folder(path = RelativePathModel.of(raw = "Home/Energy")),
        ).map { FilterTokens.tokenOf(filter = it, catalog = catalog) }

        assertEquals(
            expected = listOf(
                "category: Invoices › Energy",
                "category: Without a category",
                "tag: invoice",
                "year: 2025",
                "from: Sample Utility",
                "type: PDF",
                "folder: Home/Energy",
            ),
            actual = tokens.map { it.label },
        )
        assertEquals(expected = true, actual = tokens.all { it.filter != null })
    }

    @Test
    fun `a saved view's conditions are tokens that cannot be removed`() {
        val query = DocumentQueryModel(
            category = SyntheticCatalog.invoices.id,
            tags = TagFilterModel(all = setOf(invoice), none = setOf(TagNameModel.of(raw = "paid"))),
            dates = DateRangeModel.untilEndOf(year = 2023),
            correspondent = "Sample Utility",
        )

        val tokens = FilterTokens.savedTokens(query = query, catalog = catalog)

        assertEquals(
            expected = listOf(
                FilterTokenModel(key = "category", value = "Invoices"),
                FilterTokenModel(key = "year", value = "Until 2023"),
                FilterTokenModel(key = "from", value = "Sample Utility"),
                FilterTokenModel(key = "tag", value = "invoice"),
                FilterTokenModel(key = "not tag", value = "paid"),
            ),
            actual = tokens,
        )
    }

    @Test
    fun `typing suggests known tags, a four-digit year and correspondents, without active filters`() {
        val tags = suggest(typed = "in")
        val year = suggest(typed = "2025")
        val correspondent = suggest(typed = "UTIL")
        val active = suggest(typed = "in", active = listOf(FacetFilter.Tag(tag = invoice)))

        assertEquals(expected = listOf("tag: insurance", "tag: invoice"), actual = tags)
        assertEquals(expected = listOf("year: 2025"), actual = year)
        assertEquals(expected = listOf("from: Sample Utility"), actual = correspondent)
        assertEquals(expected = listOf("tag: insurance"), actual = active)
        assertEquals(expected = emptyList(), actual = suggest(typed = "i"))
        assertEquals(expected = emptyList(), actual = suggest(typed = "  "))
    }

    private fun suggest(typed: String, active: List<FacetFilter> = emptyList()): List<String> {
        return FilterTokens.suggestions(
            typed = typed,
            documents = catalog.documents.values,
            active = active,
            catalog = catalog,
        ).map { it.label }
    }
}
