package dms.ui.library

import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.document.InspectorViewModel
import dms.ui.document.model.MetadataDraftModel
import dms.ui.library.documents.model.SortColumn
import dms.ui.library.filter.model.ActiveFilter
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.LibraryUiStateModel
import dms.ui.library.model.YearFilterModel
import dms.ui.support.SampleLibrary
import dms.ui.support.UiTest
import kotlinx.coroutines.flow.first
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class LibraryFiltersAndSortTest {

    @Test
    fun `every active filter shows as a chip, and removing one clears only that filter`() {
        UiTest.run { app ->
            val documents = SampleLibrary.open(app = app, "letters/scan-1.txt")
            val library = app.libraryViewModel()
            val inspector = InspectorViewModel(services = app.services, session = app.session, platform = app.platform)
            inspector.addTag(document = documents.getValue(key = "letters/scan-1.txt"), raw = "tax")
            app.idle()
            val year = library.state.first { it.tags.isNotEmpty() }.years.single()
            val tag = TagNameModel.of(raw = "tax")

            library.select(selection = LibrarySelection.Tag(tag = tag))
            library.filterYear(year = YearFilterModel(year = year))
            library.search(query = " scan ")
            val all = library.state.value.filters
            library.removeFilter(filter = ActiveFilter.Year(year = YearFilterModel(year = year)))
            val withoutYear = library.state.value
            library.removeFilter(filter = ActiveFilter.Text(query = "scan", inContent = true))
            library.removeFilter(filter = ActiveFilter.Tag(tag = tag))
            val none = library.state.value

            assertEquals(
                expected = listOf(
                    ActiveFilter.Tag(tag = tag),
                    ActiveFilter.Year(year = YearFilterModel(year = year)),
                    ActiveFilter.Text(query = "scan", inContent = true),
                ),
                actual = all,
            )
            assertEquals(
                expected = listOf(ActiveFilter.Tag(tag = tag), ActiveFilter.Text(query = "scan", inContent = true)),
                actual = withoutYear.filters,
            )
            assertEquals(expected = null, actual = withoutYear.year)
            assertEquals(expected = emptyList(), actual = none.filters)
            assertEquals(expected = "", actual = none.query)
            assertEquals(expected = LibrarySelection.Folder(path = RelativePathModel.ROOT), actual = none.selection)
            assertEquals(expected = 1, actual = none.documents.size)
        }
    }

    @Test
    fun `a category is a filter, a folder is not, and the text chip says where it searches`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app, "letters/scan-1.txt")
            val library = app.libraryViewModel()
            library.newCategory()
            library.dialogs.submit(text = "Invoices")
            app.idle()
            val category = library.state.first { it.categories.isNotEmpty() }.categories.single().category

            library.select(selection = LibrarySelection.Folder(path = RelativePathModel.of(raw = "letters")))
            val folder = library.state.value.filters
            library.select(selection = LibrarySelection.Category(categoryId = category.id))
            library.contentSearch.enable(on = false)
            library.search(query = "scan")
            val filtered = library.state.first { !it.contentSearch.enabled }.filters

            assertEquals(expected = emptyList(), actual = folder)
            assertEquals(
                expected = listOf(
                    ActiveFilter.Category(categoryId = category.id, name = "Invoices"),
                    ActiveFilter.Text(query = "scan", inContent = false),
                ),
                actual = filtered,
            )
        }
    }

    @Test
    fun `a click on a column sorts by it, a second turns the order, and documents without a value stay last`() {
        UiTest.run { app ->
            val documents = SampleLibrary.open(app = app, "b.txt", "a.txt", "c.txt")
            val library = app.libraryViewModel()
            val inspector = InspectorViewModel(services = app.services, session = app.session, platform = app.platform)
            inspector.saveMetadata(
                document = documents.getValue(key = "b.txt"),
                draft = MetadataDraftModel(correspondent = "Sample Energy"),
            )
            app.idle()
            library.state.first { state -> state.documents.any { it.document.metadata.correspondent != null } }

            library.sortBy(column = SortColumn.NAME)
            val byName = namesOf(state = library.state.value)
            library.sortBy(column = SortColumn.NAME)
            val byNameDescending = namesOf(state = library.state.value)
            library.sortBy(column = SortColumn.CORRESPONDENT)
            library.sortBy(column = SortColumn.CORRESPONDENT)
            val byCorrespondentDescending = library.state.value

            assertEquals(expected = listOf("a.txt", "b.txt", "c.txt"), actual = byName)
            assertEquals(expected = listOf("c.txt", "b.txt", "a.txt"), actual = byNameDescending)
            assertEquals(
                expected = listOf("b.txt", "a.txt", "c.txt"),
                actual = namesOf(state = byCorrespondentDescending),
            )
            assertTrue(actual = byCorrespondentDescending.sort?.descending == true)
        }
    }

    private fun namesOf(state: LibraryUiStateModel): List<String> {
        return state.documents.map { it.document.path.value }
    }
}
