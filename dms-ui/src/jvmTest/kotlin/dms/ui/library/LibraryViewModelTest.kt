package dms.ui.library

import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.document.InspectorViewModel
import dms.ui.document.model.MetadataDraftModel
import dms.ui.library.drag.model.DragItem
import dms.ui.library.drag.model.DropTarget
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.LibraryUiStateModel
import dms.ui.library.model.YearFilterModel
import dms.ui.library.sidebar.model.SidebarGroup
import dms.ui.library.sidebar.model.ToggleTarget
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
    fun `files are added to the selected folder of every library folder`() {
        UiTest.run { app ->
            val library = libraryWithOneFile(app = app)

            val state = library.state.value
            assertEquals(
                expected = listOf("letters/scan-1.pdf"),
                actual = state.documents.map { it.document.path.value },
            )
            assertEquals(expected = "alpha", actual = app.read(file = app.sync.resolve("letters/scan-1.pdf")))
            assertEquals(expected = 1, actual = state.folders.single { it.path.value == "letters" }.documentCount)
        }
    }

    @Test
    fun `a category is created, assigned and used as a filter`() {
        UiTest.run { app ->
            val library = libraryWithOneFile(app = app)
            val inspector = InspectorViewModel(services = app.services, session = app.session, platform = app.platform)

            library.newCategory()
            library.dialogs.submit(text = "Invoices")
            app.idle()
            val category = library.state.first { it.categories.isNotEmpty() }.categories.single().category
            inspector.assignCategory(document = documentOf(state = library.state.value), categoryId = category.id)
            app.idle()

            library.select(selection = LibrarySelection.Category(categoryId = category.id))
            val state = library.state.first { it.uncategorizedCount == 0 }
            assertEquals(expected = 1, actual = state.documents.size)
            assertEquals(expected = "Invoices", actual = state.documents.single().categoryName)
        }
    }

    @Test
    fun `tags are added and removed`() {
        UiTest.run { app ->
            val library = libraryWithOneFile(app = app)
            val inspector = InspectorViewModel(services = app.services, session = app.session, platform = app.platform)

            inspector.addTag(document = documentOf(state = library.state.value), raw = " Tax ")
            app.idle()
            val tagged = library.state.first { it.tags.isNotEmpty() }
            assertEquals(expected = setOf(TagNameModel.of(raw = "Tax")), actual = documentOf(state = tagged).tags)

            inspector.removeTag(document = documentOf(state = tagged), tag = TagNameModel.of(raw = "tax"))
            app.idle()
            assertEquals(expected = emptySet(), actual = documentOf(state = library.state.value).tags)
        }
    }

    @Test
    fun `a rename changes the file in every folder and keeps the old name otherwise`() {
        UiTest.run { app ->
            val library = libraryWithOneFile(app = app)
            val inspector = InspectorViewModel(services = app.services, session = app.session, platform = app.platform)

            inspector.rename(document = documentOf(state = library.state.value))
            inspector.dialogs.submit(text = "a:b.pdf")
            app.idle()
            assertEquals(expected = true, actual = app.session.state.value.message?.isError)
            assertTrue(actual = Files.exists(app.primary.resolve("letters/scan-1.pdf")))

            inspector.rename(document = documentOf(state = library.state.value))
            inspector.dialogs.submit(text = "2024-01 Sample.pdf")
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
            val inspector = InspectorViewModel(services = app.services, session = app.session, platform = app.platform)
            val document = documentOf(state = library.state.value)

            inspector.saveMetadata(
                document = document,
                draft = MetadataDraftModel(title = "Sample", documentDate = "2024-02-30"),
            )
            app.idle()
            assertNull(actual = documentOf(state = library.state.value).metadata.title)

            inspector.saveMetadata(
                document = document,
                draft = MetadataDraftModel(title = "Sample", documentDate = "2024-02-01"),
            )
            app.idle()
            val saved = documentOf(state = library.state.first { documentOf(state = it).metadata.title != null })
            assertEquals(expected = LocalDate.parse("2024-02-01"), actual = saved.metadata.documentDate)
        }
    }

    @Test
    fun `the filter finds documents by name and title`() {
        UiTest.run { app ->
            val library = libraryWithOneFile(app = app)

            library.search(query = "SCAN")
            assertEquals(expected = 1, actual = library.state.value.documents.size)

            library.search(query = "nothing like it")
            assertEquals(expected = 0, actual = library.state.value.documents.size)
        }
    }

    @Test
    fun `the year filter uses the document date`() {
        UiTest.run { app ->
            val library = libraryWithOneFile(app = app)
            val inspector = InspectorViewModel(services = app.services, session = app.session, platform = app.platform)
            inspector.saveMetadata(
                document = documentOf(state = library.state.value),
                draft = MetadataDraftModel(title = "Sample", documentDate = "2019-05-01"),
            )
            app.idle()
            library.state.first { it.years == listOf(2019) }

            library.filterYear(year = YearFilterModel(year = 2020))
            val none = library.state.value.documents.size
            library.filterYear(year = YearFilterModel(year = 2020, andEarlier = true))

            assertEquals(expected = 0, actual = none)
            assertEquals(expected = 1, actual = library.state.value.documents.size)
        }
    }

    @Test
    fun `recently added shows new documents, and a folded section hides its entries`() {
        UiTest.run { app ->
            val library = libraryWithOneFile(app = app)

            library.select(selection = LibrarySelection.Recent)
            val recent = library.state.value
            library.toggle(target = ToggleTarget.Group(group = SidebarGroup.FOLDERS))

            assertEquals(expected = "Recently added", actual = recent.title)
            assertEquals(expected = 1, actual = recent.documents.size)
            assertEquals(expected = setOf(SidebarGroup.FOLDERS), actual = library.state.value.collapsedGroups)
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
            library.newCategory()
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
            val unfiled = library.state.first { documentOf(state = it).categoryId == null }
            assertEquals(expected = 1, actual = unfiled.uncategorizedCount)
        }
    }

    @Test
    fun `adding a subcategory from the menu puts it below that category, whatever is selected`() {
        UiTest.run { app ->
            val library = libraryWithOneFile(app = app)
            library.newCategory()
            library.dialogs.submit(text = "Invoices")
            app.idle()
            val invoices = library.state.first { it.categories.isNotEmpty() }.categories.single().category
            library.select(selection = LibrarySelection.Recent)

            library.newSubcategory(parent = invoices)
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
                library.newCategory()
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

    /** A library with the folder `letters` and one file in it, inspected. */
    private suspend fun libraryWithOneFile(app: TestApp): LibraryViewModel {
        app.configure()
        val library = app.libraryViewModel()
        library.newFolder()
        library.dialogs.submit(text = "letters")
        app.idle()
        val source = app.write(file = app.root.resolve("outside/scan-1.pdf"), content = "alpha")
        app.platform.nextFiles = listOf(source.toString())
        library.addFiles()
        app.idle()
        val state = library.state.first { it.documents.isNotEmpty() }
        library.inspect(documentId = state.documents.single().document.id)
        return library
    }

    private fun documentOf(state: LibraryUiStateModel): dms.domain.document.model.DocumentModel {
        return checkNotNull(value = state.inspected)
    }
}
