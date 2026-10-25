package dms.ui.library

import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.document.QuickLookViewModel
import dms.ui.document.model.MetadataDraftModel
import dms.ui.library.drag.model.DragItem
import dms.ui.library.drag.model.DropTarget
import dms.ui.library.filter.model.Facet
import dms.ui.library.filter.model.FacetFilter
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.LibraryUiStateModel
import dms.ui.library.model.YearFilterModel
import dms.ui.library.sidebar.model.SidebarGroup
import dms.ui.support.TestApp
import dms.ui.support.UiTest
import kotlinx.coroutines.flow.first
import java.nio.file.Files
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class LibraryViewModelTest {

    @Test
    fun `files are added to the filtered folder of every library folder`() {
        UiTest.run { app ->
            val library = libraryWithOneFile(app = app)

            val state = library.state.value
            assertEquals(
                expected = listOf("letters/scan-1.pdf"),
                actual = state.documents.map { it.document.path.value },
            )
            assertEquals(expected = "alpha", actual = app.read(file = app.sync.resolve("letters/scan-1.pdf")))
            assertEquals(
                expected = 1,
                actual = state.facets.getValue(key = Facet.FOLDER).single { it.label == "letters" }.count,
            )
        }
    }

    @Test
    fun `a category is created, assigned and used as a facet filter`() {
        UiTest.run { app ->
            val library = libraryWithOneFile(app = app)
            val quickLookViewModel = QuickLookViewModel(
                services = app.services,
                session = app.session,
                platform = app.platform,
            )

            library.taxonomy.newCategory(parentId = null)
            library.dialogs.submit(text = "Invoices")
            app.idle()
            val category = library.state.first { it.categories.isNotEmpty() }.categories.single().category
            quickLookViewModel.assignCategory(
                document = documentOf(state = library.state.value),
                categoryId = category.id,
            )
            app.idle()
            library.state.first { documentOf(state = it).categoryId == category.id }

            library.addFilter(filter = FacetFilter.Category(categoryId = category.id))
            val filed = library.state.value
            library.addFilter(filter = FacetFilter.Category(categoryId = null))
            val without = library.state.value

            assertEquals(expected = listOf("Invoices"), actual = filed.documents.map { it.categoryName })
            assertEquals(
                expected = listOf("folder: letters", "category: Invoices"),
                actual = filed.tokens.map { it.label },
            )
            assertEquals(expected = emptyList(), actual = without.documents)
            assertEquals(
                expected = listOf("folder: letters", "category: Without a category"),
                actual = without.tokens.map { it.label },
            )
        }
    }

    @Test
    fun `tags are added and removed`() {
        UiTest.run { app ->
            val library = libraryWithOneFile(app = app)
            val quickLookViewModel = QuickLookViewModel(
                services = app.services,
                session = app.session,
                platform = app.platform,
            )

            quickLookViewModel.addTag(document = documentOf(state = library.state.value), raw = " Tax ")
            app.idle()
            val tagged = library.state.first { it.tags.isNotEmpty() }
            assertEquals(expected = setOf(TagNameModel.of(raw = "Tax")), actual = documentOf(state = tagged).tags)

            quickLookViewModel.removeTag(document = documentOf(state = tagged), tag = TagNameModel.of(raw = "tax"))
            app.idle()
            assertEquals(expected = emptySet(), actual = documentOf(state = library.state.value).tags)
        }
    }

    @Test
    fun `a rename changes the file in every folder and keeps the old name otherwise`() {
        UiTest.run { app ->
            val library = libraryWithOneFile(app = app)
            val quickLookViewModel = QuickLookViewModel(
                services = app.services,
                session = app.session,
                platform = app.platform,
            )

            quickLookViewModel.rename(document = documentOf(state = library.state.value))
            quickLookViewModel.dialogs.submit(text = "a:b.pdf")
            app.idle()
            assertEquals(expected = true, actual = app.session.state.value.message?.isError)
            assertTrue(actual = Files.exists(app.primary.resolve("letters/scan-1.pdf")))

            quickLookViewModel.rename(document = documentOf(state = library.state.value))
            quickLookViewModel.dialogs.submit(text = "2024-01 Sample.pdf")
            app.idle()

            assertTrue(actual = Files.exists(app.primary.resolve("letters/2024-01 Sample.pdf")))
            assertTrue(actual = Files.exists(app.sync.resolve("letters/2024-01 Sample.pdf")))
            assertFalse(actual = Files.exists(app.sync.resolve("letters/scan-1.pdf")))
        }
    }

    @Test
    fun `metadata with an invalid date is refused`() {
        UiTest.run { app ->
            val library = libraryWithOneFile(app = app)
            val quickLookViewModel = QuickLookViewModel(
                services = app.services,
                session = app.session,
                platform = app.platform,
            )
            val document = documentOf(state = library.state.value)

            quickLookViewModel.saveMetadata(
                document = document,
                draft = MetadataDraftModel(title = "Sample", documentDate = "2024-02-30"),
            )
            app.idle()
            assertNull(actual = documentOf(state = library.state.value).metadata.title)

            quickLookViewModel.saveMetadata(
                document = document,
                draft = MetadataDraftModel(title = "Sample", documentDate = "2024-02-01"),
            )
            app.idle()
            val saved = documentOf(state = library.state.first { documentOf(state = it).metadata.title != null })
            assertEquals(expected = LocalDate.parse("2024-02-01"), actual = saved.metadata.documentDate)
        }
    }

    @Test
    fun `the search finds documents by name and title`() {
        UiTest.run { app ->
            val library = libraryWithOneFile(app = app)

            library.search(query = "SCAN")
            assertEquals(expected = 1, actual = library.state.value.documents.size)

            library.search(query = "nothing like it")
            assertEquals(expected = 0, actual = library.state.value.documents.size)
        }
    }

    @Test
    fun `the year facet uses the document date`() {
        UiTest.run { app ->
            val library = libraryWithOneFile(app = app)
            val quickLookViewModel = QuickLookViewModel(
                services = app.services,
                session = app.session,
                platform = app.platform,
            )
            quickLookViewModel.saveMetadata(
                document = documentOf(state = library.state.value),
                draft = MetadataDraftModel(title = "Sample", documentDate = "2019-05-01"),
            )
            app.idle()
            val years = library.state.first { state -> state.facets[Facet.YEAR].orEmpty().any { it.label == "2019" } }

            library.addFilter(filter = FacetFilter.Year(year = YearFilterModel(year = 2020)))
            val none = library.state.value.documents.size
            library.addFilter(filter = FacetFilter.Year(year = YearFilterModel(year = 2020, andEarlier = true)))

            assertEquals(expected = listOf("2019"), actual = years.facets.getValue(key = Facet.YEAR).map { it.label })
            assertEquals(expected = 0, actual = none)
            assertEquals(expected = 1, actual = library.state.value.documents.size)
            assertEquals(
                expected = listOf("folder: letters", "year: Until 2020"),
                actual = library.state.value.tokens.map { it.label },
            )
        }
    }

    @Test
    fun `a sidebar view opens without filters, and a folded section stays folded`() {
        UiTest.run { app ->
            val library = libraryWithOneFile(app = app)
            library.search(query = "scan")

            library.select(selection = LibrarySelection.Recent)
            val recent = library.state.value
            library.toggleSection(group = SidebarGroup.LIBRARY)

            assertEquals(expected = "Added this month", actual = recent.title)
            assertEquals(expected = 1, actual = recent.documents.size)
            assertEquals(expected = 1, actual = recent.counts.addedThisMonth)
            assertEquals(expected = "", actual = recent.query)
            assertEquals(expected = emptyList(), actual = recent.filters)
            assertEquals(expected = setOf(SidebarGroup.LIBRARY), actual = library.state.value.collapsedGroups)
        }
    }

    @Test
    fun `the dashboard and the palette open all documents filtered by a category, tag, folder or year`() {
        UiTest.run { app ->
            val library = libraryWithOneFile(app = app)
            val tag = TagNameModel.of(raw = "tax")

            library.show(
                selection = LibrarySelection.Tag(tag = tag),
                query = "scan",
                year = YearFilterModel(year = 2024),
            )
            val byTag = library.state.value
            library.show(selection = LibrarySelection.Folder(path = RelativePathModel.of(raw = "letters")), query = "")
            val byFolder = library.state.value

            assertEquals(expected = LibrarySelection.Folder(path = RelativePathModel.ROOT), actual = byTag.selection)
            assertEquals(expected = listOf("tag: tax", "year: 2024"), actual = byTag.tokens.map { it.label })
            assertEquals(expected = "scan", actual = byTag.query)
            assertEquals(expected = listOf("folder: letters"), actual = byFolder.tokens.map { it.label })
            assertEquals(expected = 1, actual = byFolder.documents.size)
        }
    }

    @Test
    fun `dropping a document on a folder moves it in every library folder`() {
        UiTest.run { app ->
            val library = libraryWithOneFile(app = app)

            library.drop(
                item = DragItem.DraggedDocument(document = documentOf(state = library.state.value)),
                target = DropTarget.Folder(path = RelativePathModel.ROOT),
            )
            app.idle()

            assertTrue(actual = Files.exists(app.primary.resolve("scan-1.pdf")))
            assertTrue(actual = Files.exists(app.sync.resolve("scan-1.pdf")))
            assertFalse(actual = Files.exists(app.primary.resolve("letters/scan-1.pdf")))
        }
    }

    @Test
    fun `dropping a document on a category or a tag files it there`() {
        UiTest.run { app ->
            val library = libraryWithOneFile(app = app)
            library.taxonomy.newCategory(parentId = null)
            library.dialogs.submit(text = "Invoices")
            app.idle()
            val category = library.state.first { it.categories.isNotEmpty() }.categories.single().category

            library.drop(
                item = DragItem.DraggedDocument(document = documentOf(state = library.state.value)),
                target = DropTarget.Category(categoryId = category.id),
            )
            app.idle()
            library.drop(
                item = DragItem.DraggedDocument(document = documentOf(state = library.state.value)),
                target = DropTarget.Tag(tag = TagNameModel.of(raw = "tax")),
            )
            app.idle()

            val filed = documentOf(state = library.state.first { documentOf(state = it).tags.isNotEmpty() })
            assertEquals(expected = category.id, actual = filed.categoryId)
            assertEquals(expected = setOf(TagNameModel.of(raw = "tax")), actual = filed.tags)

            library.drop(item = DragItem.DraggedDocument(document = filed), target = DropTarget.Uncategorized)
            app.idle()
            assertNull(
                actual = library.state.first { documentOf(state = it).categoryId == null }.inspected?.categoryId,
            )
        }
    }

    @Test
    fun `adding a subcategory from the menu puts it below that category, whatever is selected`() {
        UiTest.run { app ->
            val library = libraryWithOneFile(app = app)
            library.taxonomy.newCategory(parentId = null)
            library.dialogs.submit(text = "Invoices")
            app.idle()
            val invoices = library.state.first { it.categories.isNotEmpty() }.categories.single().category
            library.select(selection = LibrarySelection.Recent)

            library.taxonomy.newCategory(parentId = invoices.id)
            library.dialogs.submit(text = "Energy")
            app.idle()

            val catalog = app.services.library.catalog.first { it?.categories?.size == 2 }
            val energy = catalog?.categories?.values?.single { it.name == "Energy" }
            assertEquals(expected = invoices.id, actual = energy?.parentId)
        }
    }

    @Test
    fun `dropping a category moves it into another category and back to the top level`() {
        UiTest.run { app ->
            val library = libraryWithOneFile(app = app)
            for (name in listOf("Invoices", "Receipts")) {
                library.taxonomy.newCategory(parentId = null)
                library.dialogs.submit(text = name)
                app.idle()
            }
            val categories = library.state.first { it.categories.size == 2 }.categories.map { it.category }
            val invoices = categories.single { it.name == "Invoices" }
            val receipts = categories.single { it.name == "Receipts" }

            library.drop(
                item = DragItem.DraggedCategory(category = receipts),
                target = DropTarget.Category(categoryId = invoices.id),
            )
            app.idle()
            val nested = app.services.library.catalog.first { it?.categories?.get(receipts.id)?.parentId != null }
            assertEquals(expected = invoices.id, actual = nested?.categories?.get(receipts.id)?.parentId)

            val moved = checkNotNull(value = nested?.categories?.get(receipts.id))
            library.drop(item = DragItem.DraggedCategory(category = moved), target = DropTarget.TopLevelCategory)
            app.idle()
            val top = app.services.library.catalog.first { it?.categories?.get(receipts.id)?.parentId == null }
            assertNull(actual = top?.categories?.get(receipts.id)?.parentId)
        }
    }

    /** A library with the folder `letters` and one file in it, open in quick look. */
    private suspend fun libraryWithOneFile(app: TestApp): LibraryViewModel {
        app.configure()
        val library = app.libraryViewModel()
        app.services.documents.createFolder(path = RelativePathModel.of(raw = "letters"))
        library.select(selection = LibrarySelection.Folder(path = RelativePathModel.of(raw = "letters")))
        val source = app.write(file = app.root.resolve("outside/scan-1.pdf"), content = "alpha")
        app.platform.nextFiles = listOf(source.toString())
        library.files.addFiles()
        app.idle()
        val state = library.state.first { it.documents.isNotEmpty() }
        library.inspect(documentId = state.documents.single().document.id)
        return library
    }

    private fun documentOf(state: LibraryUiStateModel): DocumentModel {
        return checkNotNull(value = state.inspected)
    }
}
