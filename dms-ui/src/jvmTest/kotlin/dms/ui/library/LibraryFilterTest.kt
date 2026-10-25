package dms.ui.library

import dms.domain.catalog.model.CatalogModel
import dms.domain.catalog.model.CatalogRevisionModel
import dms.domain.catalog.model.LibraryIdModel
import dms.domain.document.model.ContentHashModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentMetadataModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.domain.query.model.DateRangeModel
import dms.domain.query.model.DocumentQueryModel
import dms.domain.query.model.SavedSearchIdModel
import dms.domain.query.model.SavedSearchModel
import dms.domain.query.model.TagFilterModel
import dms.domain.settings.model.CatalogSettingsModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.CategoryModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.LibraryViewStateModel
import dms.ui.library.model.YearFilterModel
import java.time.Duration
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import java.time.ZoneOffset
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class LibraryFilterTest {

    private val now = Instant.parse("2024-06-30T12:00:00Z")
    private val invoices = CategoryModel(id = CategoryIdModel(value = "invoices"), name = "Invoices")
    private val energy = CategoryModel(id = CategoryIdModel(value = "energy"), name = "Energy", parentId = invoices.id)
    private val tax = TagNameModel.of(raw = "Tax")

    @Test
    fun `a folder shows every document below it, sorted by path ignoring case`() {
        val catalog = catalogOf(
            document(id = "1", path = "letters/B.pdf"),
            document(id = "2", path = "archive/c.pdf"),
            document(id = "3", path = "letters/a.pdf"),
            document(id = "4", path = "letters-old/d.pdf"),
        )

        val letters = pathsOf(catalog = catalog, selection = LibrarySelection.Folder(path = path(raw = "letters")))
        val all = pathsOf(catalog = catalog, selection = LibrarySelection.Folder(path = RelativePathModel.ROOT))

        assertEquals(expected = listOf("letters/a.pdf", "letters/B.pdf"), actual = letters)
        assertEquals(
            expected = listOf("archive/c.pdf", "letters-old/d.pdf", "letters/a.pdf", "letters/B.pdf"),
            actual = all,
        )
    }

    @Test
    fun `a category includes its subcategories and uncategorized shows only documents without one`() {
        val catalog = catalogOf(
            document(id = "1", path = "a.pdf", categoryId = invoices.id),
            document(id = "2", path = "b.pdf", categoryId = energy.id),
            document(id = "3", path = "c.pdf"),
        )

        val parent = pathsOf(catalog = catalog, selection = LibrarySelection.Category(categoryId = invoices.id))
        val child = pathsOf(catalog = catalog, selection = LibrarySelection.Category(categoryId = energy.id))
        val none = pathsOf(catalog = catalog, selection = LibrarySelection.Uncategorized)

        assertEquals(expected = listOf("a.pdf", "b.pdf"), actual = parent)
        assertEquals(expected = listOf("b.pdf"), actual = child)
        assertEquals(expected = listOf("c.pdf"), actual = none)
    }

    @Test
    fun `a tag shows the documents with it, whatever its casing`() {
        val catalog = catalogOf(
            document(id = "1", path = "a.pdf", tags = setOf(tax)),
            document(id = "2", path = "b.pdf"),
        )

        val tagged = pathsOf(catalog = catalog, selection = LibrarySelection.Tag(tag = TagNameModel.of(raw = "TAX")))

        assertEquals(expected = listOf("a.pdf"), actual = tagged)
    }

    @Test
    fun `recently added reaches back 30 days, newest first`() {
        val catalog = catalogOf(
            document(id = "1", path = "a.pdf", addedAt = now.minus(Duration.ofDays(10))),
            document(id = "2", path = "b.pdf", addedAt = now.minus(Duration.ofDays(30))),
            document(id = "3", path = "c.pdf", addedAt = now.minus(Duration.ofDays(30)).minusSeconds(1)),
            document(id = "4", path = "d.pdf", addedAt = now.minus(Duration.ofDays(1))),
        )

        val recent = pathsOf(catalog = catalog, selection = LibrarySelection.Recent)

        assertEquals(expected = listOf("d.pdf", "a.pdf", "b.pdf"), actual = recent)
    }

    @Test
    fun `the year is the one of the document date, or of when it was added in the given zone`() {
        val dated = document(
            id = "1",
            path = "a.pdf",
            addedAt = Instant.parse("2024-02-01T00:00:00Z"),
            documentDate = LocalDate.parse("2019-05-01"),
        )
        val undated = document(id = "2", path = "b.pdf", addedAt = Instant.parse("2020-12-31T23:30:00Z"))
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
            document(id = "1", path = "a.pdf", documentDate = LocalDate.parse("2019-05-01")),
            document(id = "2", path = "b.pdf", addedAt = Instant.parse("2020-03-01T00:00:00Z")),
            document(id = "3", path = "c.pdf", documentDate = LocalDate.parse("2021-01-01")),
        )
        val root = LibrarySelection.Folder(path = RelativePathModel.ROOT)

        val single = pathsOf(catalog = catalog, selection = root, year = YearFilterModel(year = 2020))
        val earlier = pathsOf(
            catalog = catalog,
            selection = root,
            year = YearFilterModel(year = 2020, andEarlier = true),
        )

        assertEquals(expected = listOf("b.pdf"), actual = single)
        assertEquals(expected = listOf("a.pdf", "b.pdf"), actual = earlier)
    }

    @Test
    fun `the text filter matches name, title, correspondent and tags and adds the text matches of the selection`() {
        val catalog = catalogOf(
            document(id = "1", path = "letters/a.pdf", title = "Water bill"),
            document(id = "2", path = "letters/b.pdf", correspondent = "Sample Utility"),
            document(id = "3", path = "letters/c.pdf", tags = setOf(tax)),
            document(id = "4", path = "letters/d.pdf"),
            document(id = "5", path = "archive/e.pdf"),
        )
        val letters = LibrarySelection.Folder(path = path(raw = "letters"))
        val textMatches = setOf(DocumentIdModel(value = "4"), DocumentIdModel(value = "5"))

        val byTitle = pathsOf(catalog = catalog, selection = letters, query = "  WATER ")
        val byCorrespondent = pathsOf(catalog = catalog, selection = letters, query = "utility")
        val byTag = pathsOf(catalog = catalog, selection = letters, query = "tax")
        val withText = pathsOf(catalog = catalog, selection = letters, query = "receipt", textMatches = textMatches)

        assertEquals(expected = listOf("letters/a.pdf"), actual = byTitle)
        assertEquals(expected = listOf("letters/b.pdf"), actual = byCorrespondent)
        assertEquals(expected = listOf("letters/c.pdf"), actual = byTag)
        assertEquals(expected = listOf("letters/d.pdf"), actual = withText)
    }

    @Test
    fun `query matches narrow what the selection shows`() {
        val catalog = catalogOf(document(id = "1", path = "a.pdf"), document(id = "2", path = "b.pdf"))
        val root = LibrarySelection.Folder(path = RelativePathModel.ROOT)

        val narrowed = pathsOf(catalog = catalog, selection = root, queryMatches = setOf(DocumentIdModel(value = "2")))
        val nothing = pathsOf(catalog = catalog, selection = root, queryMatches = emptySet())

        assertEquals(expected = listOf("b.pdf"), actual = narrowed)
        assertEquals(expected = emptyList(), actual = nothing)
    }

    @Test
    fun `a smart folder shows its saved query, the year filter replaces its dates, and a deleted one shows nothing`() {
        val saved = SavedSearchModel(
            id = SavedSearchIdModel(value = "tax-2019"),
            name = "Tax 2019",
            query = DocumentQueryModel(
                tags = TagFilterModel(all = setOf(tax)),
                dates = DateRangeModel.year(year = 2019),
            ),
        )
        val catalog = catalogOf(
            document(id = "1", path = "a.pdf", tags = setOf(tax), documentDate = LocalDate.parse("2019-05-01")),
            document(id = "2", path = "b.pdf", tags = setOf(tax), documentDate = LocalDate.parse("2020-05-01")),
            document(id = "3", path = "c.pdf", documentDate = LocalDate.parse("2019-05-01")),
            searches = listOf(saved),
        )
        val smartFolder = LibrarySelection.SmartFolder(id = saved.id)
        val deleted = LibrarySelection.SmartFolder(id = SavedSearchIdModel(value = "deleted"))

        val asSaved = pathsOf(catalog = catalog, selection = smartFolder)
        val otherYear = pathsOf(catalog = catalog, selection = smartFolder, year = YearFilterModel(year = 2020))

        assertEquals(expected = listOf("a.pdf"), actual = asSaved)
        assertEquals(expected = listOf("b.pdf"), actual = otherYear)
        assertEquals(expected = emptyList(), actual = pathsOf(catalog = catalog, selection = deleted))
        assertNull(actual = LibraryFilter.queryOf(selection = deleted, year = null, now = now, catalog = catalog))
    }

    @Test
    fun `the title names the selection and falls back when the catalog no longer has it`() {
        val saved = SavedSearchModel(
            id = SavedSearchIdModel(value = "s"),
            name = "Untagged",
            query = DocumentQueryModel(),
        )
        val catalog = catalogOf(searches = listOf(saved))
        val gone = CategoryIdModel(value = "gone")

        val titles = listOf(
            LibrarySelection.Folder(path = RelativePathModel.ROOT),
            LibrarySelection.Folder(path = path(raw = "letters/2024")),
            LibrarySelection.Category(categoryId = energy.id),
            LibrarySelection.Category(categoryId = gone),
            LibrarySelection.Uncategorized,
            LibrarySelection.Tag(tag = tax),
            LibrarySelection.Recent,
            LibrarySelection.SmartFolder(id = saved.id),
            LibrarySelection.SmartFolder(id = SavedSearchIdModel(value = "deleted")),
        ).map { LibraryFilter.titleOf(selection = it, catalog = catalog) }

        assertEquals(
            expected = listOf(
                "All documents",
                "2024",
                "Energy",
                "Category",
                "Uncategorized",
                "#Tax",
                "Recently added",
                "Untagged",
                "Smart folder",
            ),
            actual = titles,
        )
    }

    private fun pathsOf(
        catalog: CatalogModel,
        selection: LibrarySelection,
        year: YearFilterModel? = null,
        query: String = "",
        textMatches: Set<DocumentIdModel> = emptySet(),
        queryMatches: Set<DocumentIdModel>? = null,
    ): List<String> {
        return LibraryFilter.documents(
            catalog = catalog,
            view = LibraryViewStateModel(selection = selection, query = query, year = year),
            now = now,
            zone = ZoneOffset.UTC,
            textMatches = textMatches,
            queryMatches = queryMatches,
        ).map { it.path.value }
    }

    private fun catalogOf(
        vararg documents: DocumentModel,
        searches: List<SavedSearchModel> = emptyList(),
    ): CatalogModel {
        return CatalogModel(
            libraryId = LibraryIdModel(value = "library"),
            revision = CatalogRevisionModel.INITIAL,
            documents = documents.associateBy { it.id },
            categories = listOf(invoices, energy).associateBy { it.id },
            tags = documents.flatMap { it.tags }.toSet(),
            settings = CatalogSettingsModel(savedSearches = searches),
        )
    }

    private fun document(
        id: String,
        path: String,
        addedAt: Instant = Instant.parse("2024-01-15T00:00:00Z"),
        documentDate: LocalDate? = null,
        title: String? = null,
        correspondent: String? = null,
        categoryId: CategoryIdModel? = null,
        tags: Set<TagNameModel> = emptySet(),
    ): DocumentModel {
        return DocumentModel(
            id = DocumentIdModel(value = id),
            path = path(raw = path),
            contentHash = ContentHashModel(value = id.padStart(length = 64, padChar = '0')),
            size = 1,
            addedAt = addedAt,
            metadata = DocumentMetadataModel(title = title, documentDate = documentDate, correspondent = correspondent),
            categoryId = categoryId,
            tags = tags,
        )
    }

    private fun path(raw: String): RelativePathModel {
        return RelativePathModel.of(raw = raw)
    }
}
