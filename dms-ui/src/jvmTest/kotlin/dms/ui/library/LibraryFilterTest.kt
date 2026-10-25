package dms.ui.library

import dms.domain.catalog.model.CatalogModel
import dms.domain.document.DocumentKind
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.domain.query.model.DateRangeModel
import dms.domain.query.model.DocumentQueryModel
import dms.domain.query.model.SavedSearchIdModel
import dms.domain.query.model.SavedSearchModel
import dms.domain.query.model.TagFilterModel
import dms.domain.settings.model.CatalogSettingsModel
import dms.domain.settings.model.InboxSettingsModel
import dms.domain.settings.model.LibraryLayoutModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.library.filter.model.FacetFilter
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.LibraryViewStateModel
import dms.ui.library.model.YearFilterModel
import dms.ui.support.SyntheticCatalog
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import java.time.ZoneOffset
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class LibraryFilterTest {

    private val now = Instant.parse("2024-06-30T12:00:00Z")
    private val all = LibrarySelection.Folder(path = RelativePathModel.ROOT)
    private val invoices = SyntheticCatalog.invoices
    private val energy = SyntheticCatalog.energy
    private val tax = TagNameModel.of(raw = "Tax")

    @Test
    fun `a folder shows every document below it, sorted by path ignoring case`() {
        val catalog = catalogOf(
            SyntheticCatalog.document(id = "1", path = "letters/B.pdf"),
            SyntheticCatalog.document(id = "2", path = "archive/c.pdf"),
            SyntheticCatalog.document(id = "3", path = "letters/a.pdf"),
            SyntheticCatalog.document(id = "4", path = "letters-old/d.pdf"),
        )

        val letters = pathsOf(catalog = catalog, filters = listOf(FacetFilter.Folder(path = path(raw = "letters"))))
        val everything = pathsOf(catalog = catalog)

        assertEquals(expected = listOf("letters/a.pdf", "letters/B.pdf"), actual = letters)
        assertEquals(
            expected = listOf("archive/c.pdf", "letters-old/d.pdf", "letters/a.pdf", "letters/B.pdf"),
            actual = everything,
        )
    }

    @Test
    fun `a category includes its subcategories and without a category shows only documents without one`() {
        val catalog = catalogOf(
            SyntheticCatalog.document(id = "1", path = "a.pdf", categoryId = invoices.id),
            SyntheticCatalog.document(id = "2", path = "b.pdf", categoryId = energy.id),
            SyntheticCatalog.document(id = "3", path = "c.pdf"),
        )

        val parent = pathsOf(catalog = catalog, filters = listOf(FacetFilter.Category(categoryId = invoices.id)))
        val child = pathsOf(catalog = catalog, filters = listOf(FacetFilter.Category(categoryId = energy.id)))
        val none = pathsOf(catalog = catalog, filters = listOf(FacetFilter.Category(categoryId = null)))

        assertEquals(expected = listOf("a.pdf", "b.pdf"), actual = parent)
        assertEquals(expected = listOf("b.pdf"), actual = child)
        assertEquals(expected = listOf("c.pdf"), actual = none)
    }

    @Test
    fun `facet filters of different facets narrow each other`() {
        val catalog = catalogOf(
            SyntheticCatalog.document(id = "1", path = "a.pdf", tags = setOf(tax), correspondent = "Sample Utility"),
            SyntheticCatalog.document(id = "2", path = "b.png", tags = setOf(tax), correspondent = "sample utility"),
            SyntheticCatalog.document(id = "3", path = "c.pdf", tags = setOf(tax), correspondent = "Other Sender"),
            SyntheticCatalog.document(id = "4", path = "d.pdf", correspondent = "Sample Utility"),
        )

        val shown = pathsOf(
            catalog = catalog,
            filters = listOf(
                FacetFilter.Tag(tag = TagNameModel.of(raw = "TAX")),
                FacetFilter.Correspondent(name = "Sample Utility"),
                FacetFilter.Type(kind = DocumentKind.PDF),
            ),
        )

        assertEquals(expected = listOf("a.pdf"), actual = shown)
    }

    @Test
    fun `added this month starts on the first day of the calendar month, newest first`() {
        val catalog = catalogOf(
            SyntheticCatalog.document(id = "1", path = "a.pdf", addedAt = Instant.parse("2024-06-15T00:00:00Z")),
            SyntheticCatalog.document(id = "2", path = "b.pdf", addedAt = Instant.parse("2024-06-01T00:00:00Z")),
            SyntheticCatalog.document(id = "3", path = "c.pdf", addedAt = Instant.parse("2024-05-31T23:59:59Z")),
            SyntheticCatalog.document(id = "4", path = "d.pdf", addedAt = Instant.parse("2024-06-29T00:00:00Z")),
        )

        val recent = pathsOf(catalog = catalog, selection = LibrarySelection.Recent)

        assertEquals(expected = listOf("d.pdf", "a.pdf", "b.pdf"), actual = recent)
    }

    @Test
    fun `the inbox shows the documents in the inbox folder while it is on, and nothing while it is off`() {
        val documents = arrayOf(
            SyntheticCatalog.document(id = "1", path = "Inbox/scan.pdf"),
            SyntheticCatalog.document(id = "2", path = "letters/a.pdf"),
        )
        val on = catalogOf(*documents, inbox = InboxSettingsModel(enabled = true))
        val off = catalogOf(*documents, inbox = InboxSettingsModel(enabled = false))

        assertEquals(
            expected = listOf("Inbox/scan.pdf"),
            actual = pathsOf(catalog = on, selection = LibrarySelection.Inbox),
        )
        assertEquals(expected = emptyList(), actual = pathsOf(catalog = off, selection = LibrarySelection.Inbox))
    }

    @Test
    fun `needs review shows only the documents with a pending suggestion, and the folder view lists none`() {
        val catalog = catalogOf(
            SyntheticCatalog.document(id = "1", path = "a.pdf"),
            SyntheticCatalog.document(id = "2", path = "b.pdf"),
        )

        val review = pathsOf(
            catalog = catalog,
            selection = LibrarySelection.NeedsReview,
            reviewIds = setOf(DocumentIdModel(value = "2")),
        )
        val folders = pathsOf(catalog = catalog, selection = LibrarySelection.Folders)

        assertEquals(expected = listOf("b.pdf"), actual = review)
        assertEquals(expected = emptyList(), actual = folders)
    }

    @Test
    fun `the year is the one of the document date, or of when it was added in the given zone`() {
        val dated = SyntheticCatalog.document(
            id = "1",
            path = "a.pdf",
            addedAt = Instant.parse("2024-02-01T00:00:00Z"),
            documentDate = LocalDate.parse("2019-05-01"),
        )
        val undated = SyntheticCatalog.document(
            id = "2",
            path = "b.pdf",
            addedAt = Instant.parse("2020-12-31T23:30:00Z"),
        )
        val berlin = ZoneId.of("Europe/Berlin")

        assertEquals(expected = 2019, actual = LibraryFilter.yearOf(document = dated, zone = ZoneOffset.UTC))
        assertEquals(expected = 2020, actual = LibraryFilter.yearOf(document = undated, zone = ZoneOffset.UTC))
        assertEquals(expected = 2021, actual = LibraryFilter.yearOf(document = undated, zone = berlin))
        assertEquals(
            expected = listOf(2021, 2019),
            actual = LibraryFilter.years(documents = listOf(dated, undated, dated), zone = berlin),
        )
    }

    @Test
    fun `the year filter selects one year or that year and every earlier one`() {
        val catalog = catalogOf(
            SyntheticCatalog.document(id = "1", path = "a.pdf", documentDate = LocalDate.parse("2019-05-01")),
            SyntheticCatalog.document(id = "2", path = "b.pdf", addedAt = Instant.parse("2020-03-01T00:00:00Z")),
            SyntheticCatalog.document(id = "3", path = "c.pdf", documentDate = LocalDate.parse("2021-01-01")),
        )

        val single = pathsOf(catalog = catalog, filters = listOf(FacetFilter.Year(year = YearFilterModel(year = 2020))))
        val earlier = pathsOf(
            catalog = catalog,
            filters = listOf(FacetFilter.Year(year = YearFilterModel(year = 2020, andEarlier = true))),
        )

        assertEquals(expected = listOf("b.pdf"), actual = single)
        assertEquals(expected = listOf("a.pdf", "b.pdf"), actual = earlier)
    }

    @Test
    fun `the search text matches name, title, correspondent and tags and adds the text matches of the view`() {
        val catalog = catalogOf(
            SyntheticCatalog.document(id = "1", path = "letters/a.pdf", title = "Water bill"),
            SyntheticCatalog.document(id = "2", path = "letters/b.pdf", correspondent = "Sample Utility"),
            SyntheticCatalog.document(id = "3", path = "letters/c.pdf", tags = setOf(tax)),
            SyntheticCatalog.document(id = "4", path = "letters/d.pdf"),
            SyntheticCatalog.document(id = "5", path = "archive/e.pdf"),
        )
        val letters = listOf(FacetFilter.Folder(path = path(raw = "letters")))
        val textMatches = setOf(DocumentIdModel(value = "4"), DocumentIdModel(value = "5"))

        val byTitle = pathsOf(catalog = catalog, filters = letters, query = "  WATER ")
        val byCorrespondent = pathsOf(catalog = catalog, filters = letters, query = "utility")
        val byTag = pathsOf(catalog = catalog, filters = letters, query = "tax")
        val withText = pathsOf(catalog = catalog, filters = letters, query = "receipt", textMatches = textMatches)

        assertEquals(expected = listOf("letters/a.pdf"), actual = byTitle)
        assertEquals(expected = listOf("letters/b.pdf"), actual = byCorrespondent)
        assertEquals(expected = listOf("letters/c.pdf"), actual = byTag)
        assertEquals(expected = listOf("letters/d.pdf"), actual = withText)
    }

    @Test
    fun `a saved view shows its query, narrowed by facet filters, and a deleted one shows nothing`() {
        val saved = SavedSearchModel(
            id = SavedSearchIdModel(value = "tax-2019"),
            name = "Tax 2019",
            query = DocumentQueryModel(
                tags = TagFilterModel(all = setOf(tax)),
                dates = DateRangeModel.year(year = 2019),
            ),
        )
        val catalog = catalogOf(
            SyntheticCatalog.document(
                id = "1",
                path = "a.pdf",
                tags = setOf(tax),
                documentDate = LocalDate.parse("2019-05-01"),
            ),
            SyntheticCatalog.document(
                id = "2",
                path = "b.pdf",
                tags = setOf(tax),
                documentDate = LocalDate.parse("2020-05-01"),
            ),
            SyntheticCatalog.document(id = "3", path = "c.pdf", documentDate = LocalDate.parse("2019-05-01")),
            SyntheticCatalog.document(
                id = "4",
                path = "d.png",
                tags = setOf(tax),
                documentDate = LocalDate.parse("2019-08-01"),
            ),
            searches = listOf(saved),
        )
        val view = LibrarySelection.SmartFolder(id = saved.id)
        val deleted = LibrarySelection.SmartFolder(id = SavedSearchIdModel(value = "deleted"))

        val asSaved = pathsOf(catalog = catalog, selection = view)
        val otherYear = pathsOf(
            catalog = catalog,
            selection = view,
            filters = listOf(FacetFilter.Year(year = YearFilterModel(year = 2020))),
        )
        val onlyPdf = pathsOf(
            catalog = catalog,
            selection = view,
            filters = listOf(FacetFilter.Type(kind = DocumentKind.PDF)),
        )

        assertEquals(expected = listOf("a.pdf", "d.png"), actual = asSaved)
        assertEquals(expected = listOf("b.pdf"), actual = otherYear)
        assertEquals(expected = listOf("a.pdf"), actual = onlyPdf)
        assertEquals(expected = emptyList(), actual = pathsOf(catalog = catalog, selection = deleted))
        assertNull(
            actual = LibraryFilter.queryOf(
                selection = deleted,
                filters = emptyList(),
                now = now,
                zone = ZoneOffset.UTC,
                catalog = catalog,
            ),
        )
    }

    @Test
    fun `the title names the view and falls back when the catalog no longer has it`() {
        val saved = SavedSearchModel(
            id = SavedSearchIdModel(value = "s"),
            name = "Untagged",
            query = DocumentQueryModel(),
        )
        val catalog = catalogOf(searches = listOf(saved))

        val titles = listOf(
            all,
            LibrarySelection.Inbox,
            LibrarySelection.NeedsReview,
            LibrarySelection.Recent,
            LibrarySelection.Folders,
            LibrarySelection.SmartFolder(id = saved.id),
            LibrarySelection.SmartFolder(id = SavedSearchIdModel(value = "deleted")),
        ).map { LibraryFilter.titleOf(selection = it, catalog = catalog) }

        assertEquals(
            expected = listOf(
                "All documents",
                "Inbox",
                "Needs review",
                "Added this month",
                "Folders",
                "Untagged",
                "Saved view",
            ),
            actual = titles,
        )
    }

    private fun pathsOf(
        catalog: CatalogModel,
        selection: LibrarySelection = all,
        filters: List<FacetFilter> = emptyList(),
        query: String = "",
        textMatches: Set<DocumentIdModel> = emptySet(),
        reviewIds: Set<DocumentIdModel> = emptySet(),
    ): List<String> {
        return LibraryFilter.documents(
            catalog = catalog,
            view = LibraryViewStateModel(selection = selection, filters = filters, query = query),
            now = now,
            zone = ZoneOffset.UTC,
            textMatches = textMatches,
            reviewIds = reviewIds,
        ).map { it.path.value }
    }

    private fun catalogOf(
        vararg documents: DocumentModel,
        searches: List<SavedSearchModel> = emptyList(),
        inbox: InboxSettingsModel = InboxSettingsModel(),
    ): CatalogModel {
        return SyntheticCatalog.catalog(
            documents = documents,
            settings = CatalogSettingsModel(savedSearches = searches, layout = LibraryLayoutModel(inbox = inbox)),
        )
    }

    private fun path(raw: String): RelativePathModel {
        return RelativePathModel.of(raw = raw)
    }
}
