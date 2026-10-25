package dms.ui.library

import dms.domain.document.DocumentKind
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.document.QuickLookViewModel
import dms.ui.document.model.MetadataDraftModel
import dms.ui.library.documents.model.SortColumn
import dms.ui.library.filter.model.Facet
import dms.ui.library.filter.model.FacetFilter
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.LibraryUiStateModel
import dms.ui.support.SampleLibrary
import dms.ui.support.UiTest
import kotlinx.coroutines.flow.first
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class LibraryFiltersAndSortTest {

    @Test
    fun `every facet filter shows as a token, and removing one clears only that filter`() {
        UiTest.run { app ->
            val documents = SampleLibrary.open(app = app, "letters/scan-1.txt")
            val library = app.libraryViewModel()
            val quickLookViewModel = QuickLookViewModel(
                services = app.services,
                session = app.session,
                platform = app.platform,
            )
            quickLookViewModel.addTag(document = documents.getValue(key = "letters/scan-1.txt"), raw = "tax")
            app.idle()
            val year = library.state.first { it.tags.isNotEmpty() }.facets.getValue(key = Facet.YEAR).single().filter
            val tag = FacetFilter.Tag(tag = TagNameModel.of(raw = "tax"))

            library.addFilter(filter = tag)
            library.addFilter(filter = year)
            library.search(query = " scan ")
            val all = library.state.value
            library.removeFilter(filter = year)
            val withoutYear = library.state.value
            library.clearFilters()
            val none = library.state.value

            assertEquals(expected = listOf(tag, year), actual = all.filters)
            assertEquals(expected = 2, actual = all.tokens.size)
            assertEquals(expected = listOf("tag: tax"), actual = withoutYear.tokens.map { it.label })
            assertEquals(expected = " scan ", actual = withoutYear.query)
            assertEquals(expected = emptyList(), actual = none.tokens)
            assertEquals(expected = "", actual = none.query)
            assertEquals(expected = LibrarySelection.Folder(path = RelativePathModel.ROOT), actual = none.selection)
            assertEquals(expected = 1, actual = none.documents.size)
        }
    }

    @Test
    fun `a saved view shows its own conditions as tokens that stay, and facet filters on top`() {
        UiTest.run { app ->
            val documents = SampleLibrary.open(app = app, "letters/scan-1.txt", "letters/scan-2.txt")
            val library = app.libraryViewModel()
            library.addFilter(filter = FacetFilter.Folder(path = RelativePathModel.of(raw = "letters")))
            library.smartFolders.saveCurrent()
            library.dialogs.submit(text = "Letters")
            app.idle()
            val saved = library.state.first { it.smartFolders.isNotEmpty() }.smartFolders.single()
            val quickLookViewModel = QuickLookViewModel(
                services = app.services,
                session = app.session,
                platform = app.platform,
            )
            quickLookViewModel.addTag(document = documents.getValue(key = "letters/scan-2.txt"), raw = "tax")
            app.idle()

            library.select(selection = LibrarySelection.SmartFolder(id = saved.id))
            library.addFilter(filter = FacetFilter.Tag(tag = TagNameModel.of(raw = "tax")))
            val state = library.state.first { it.documents.size == 1 }

            assertEquals(expected = listOf("folder: letters", "tag: tax"), actual = state.tokens.map { it.label })
            assertEquals(expected = listOf(false, true), actual = state.tokens.map { it.filter != null })
            assertEquals(expected = "letters/scan-2.txt", actual = state.documents.single().document.path.value)
        }
    }

    @Test
    fun `a type filter or needs review cannot be saved as a view, and says why`() {
        UiTest.run { app ->
            SampleLibrary.open(app = app, "letters/scan-1.txt")
            val library = app.libraryViewModel()

            library.addFilter(filter = FacetFilter.Type(kind = DocumentKind.OTHER))
            val typed = library.state.value
            library.select(selection = LibrarySelection.NeedsReview)
            val review = library.state.value
            library.select(selection = LibrarySelection.Inbox)

            assertEquals(
                expected = "A saved view or an export cannot keep the type filter yet",
                actual = typed.queryProblem,
            )
            assertEquals(expected = "Needs review cannot be saved or exported as a view", actual = review.queryProblem)
            assertEquals(expected = null, actual = library.state.value.queryProblem)
        }
    }

    @Test
    fun `a click on a column sorts by it, a second turns the order, and documents without a value stay last`() {
        UiTest.run { app ->
            val documents = SampleLibrary.open(app = app, "b.txt", "a.txt", "c.txt")
            val library = app.libraryViewModel()
            val quickLookViewModel = QuickLookViewModel(
                services = app.services,
                session = app.session,
                platform = app.platform,
            )
            quickLookViewModel.saveMetadata(
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
