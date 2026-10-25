package dms.ui.library.selection

import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.library.LibraryViewModel
import dms.ui.library.drag.model.DragItem
import dms.ui.library.drag.model.DropTarget
import dms.ui.library.model.LibrarySelection
import dms.ui.support.SampleImages
import dms.ui.support.SampleLibrary
import dms.ui.support.TestApp
import dms.ui.support.UiTest
import kotlinx.coroutines.flow.first
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class BulkActionsTest {

    @Test
    fun `the ticked documents move into the folder picked in the picker, in every library folder`() {
        UiTest.run { app ->
            val (library, ids) = libraryWithTicks(app = app, "a.txt", "b.txt")

            library.bulk.move()
            val picker = assertNotNull(actual = library.bulk.picker.state.value)
            val rows = library.bulk.pickerFolders(expanded = emptySet()).map { it.path.value }
            library.bulk.picker.select(path = RelativePathModel.of(raw = "archive"))
            library.bulk.picker.confirm()
            app.idle()

            assertEquals(expected = "Move 2 documents to…", actual = picker.title)
            assertEquals(expected = listOf("archive"), actual = rows)
            for (name in listOf("a.txt", "b.txt")) {
                assertTrue(actual = Files.exists(app.primary.resolve("archive/$name")))
                assertTrue(actual = Files.exists(app.sync.resolve("archive/$name")))
                assertFalse(actual = Files.exists(app.primary.resolve(name)))
            }
            assertTrue(actual = Files.exists(app.primary.resolve("c.txt")))
            assertEquals(
                expected = emptySet(),
                actual = library.state.first { it.selectedDocuments.isEmpty() }.selectedDocuments,
            )
            assertEquals(expected = 2, actual = ids.size)
        }
    }

    @Test
    fun `dragging a ticked document moves the whole selection, an unticked one moves alone`() {
        UiTest.run { app ->
            val (library, _) = libraryWithTicks(app = app, "a.txt", "b.txt")
            val documents = library.state.value.documents.associate { it.document.path.value to it.document }
            val archive = DropTarget.Folder(path = RelativePathModel.of(raw = "archive"))

            library.drop(
                item = DragItem.DraggedDocument(document = documents.getValue(key = "a.txt")),
                target = archive,
            )
            app.idle()
            library.drop(
                item = DragItem.DraggedDocument(document = documents.getValue(key = "c.txt")),
                target = archive,
            )
            app.idle()

            val paths = SampleLibrary.catalog(app = app).documents.values.map { it.path.value }.sorted()
            assertEquals(
                expected = listOf("archive/a.txt", "archive/b.txt", "archive/c.txt", "archive/old.txt"),
                actual = paths,
            )
        }
    }

    @Test
    fun `trash asks first, naming every ticked document, and only then moves them to the trash`() {
        UiTest.run { app ->
            val (library, _) = libraryWithTicks(app = app, "a.txt", "b.txt")

            library.bulk.trash()
            val question = assertNotNull(actual = library.dialogs.confirmation.value)
            val untouched = Files.exists(app.primary.resolve("a.txt"))
            library.dialogs.confirm()
            app.idle()

            assertEquals(expected = "Move 2 documents to the trash?", actual = question.title)
            assertEquals(expected = listOf("a.txt", "b.txt"), actual = question.affected)
            assertTrue(actual = untouched)
            assertFalse(actual = Files.exists(app.primary.resolve("a.txt")))
            assertFalse(actual = Files.exists(app.sync.resolve("b.txt")))
            assertEquals(
                expected = listOf("archive/old.txt", "c.txt"),
                actual = SampleLibrary.catalog(app = app).documents.values.map { it.path.value }.sorted(),
            )
        }
    }

    @Test
    fun `the ticked documents get a category and a tag`() {
        UiTest.run { app ->
            val (library, ids) = libraryWithTicks(app = app, "a.txt", "b.txt")
            library.taxonomy.newCategory(parentId = null)
            library.dialogs.submit(text = "Invoices")
            app.idle()
            val category = library.state.first { it.categories.isNotEmpty() }.categories.single().category

            library.bulk.assignCategory(categoryId = category.id)
            app.idle()
            ids.forEach { library.documentSelection.toggle(documentId = it) }
            library.bulk.addTag(tag = TagNameModel.of(raw = "tax"))
            app.idle()

            val catalog = app.services.library.catalog.first { catalog ->
                ids.all { catalog?.documents?.get(it)?.tags?.isNotEmpty() == true }
            }
            val ticked = ids.map { checkNotNull(value = catalog?.documents?.get(it)) }
            assertEquals(expected = listOf(category.id, category.id), actual = ticked.map { it.categoryId })
            assertEquals(expected = setOf(TagNameModel.of(raw = "tax")), actual = ticked.flatMap { it.tags }.toSet())
            val other = catalog?.documents?.values?.single { it.path.value == "c.txt" }
            assertEquals(expected = null, actual = other?.categoryId)
        }
    }

    @Test
    fun `tag with AI only adds suggestions, which then wait under needs review`() {
        UiTest.run { app ->
            app.configure()
            val library = app.libraryViewModel()
            app.platform.nextFiles = listOf(
                SampleImages.write(file = app.root.resolve("outside/scan-7.png")).toString(),
            )
            library.files.addFilesToTopLevel()
            app.idle()
            val document = library.state.first { it.documents.isNotEmpty() }.documents.single().document
            app.ai.suggestion = "Invoice"

            library.documentSelection.toggle(documentId = document.id)
            library.bulk.tagWithAi()
            app.idle()
            val counted = library.state.first { it.counts.needsReview == 1 }
            library.select(selection = LibrarySelection.NeedsReview)

            assertEquals(expected = emptySet(), actual = counted.selectedDocuments)
            assertEquals(expected = emptySet(), actual = SampleLibrary.document(app = app, id = document.id).tags)
            assertEquals(expected = listOf(document.id), actual = library.state.value.documents.map { it.document.id })
        }
    }

    @Test
    fun `select all ticks every shown document`() {
        UiTest.run { app ->
            val (library, _) = libraryWithTicks(app = app)

            library.documentSelection.selectAll()

            assertEquals(
                expected = 4,
                actual = library.state.first { it.selectedDocuments.size == 4 }.selectedDocuments.size,
            )
        }
    }

    /** A library with `a.txt`, `b.txt`, `c.txt` and `archive/old.txt`, and [ticked] of them ticked. */
    private suspend fun libraryWithTicks(
        app: TestApp,
        vararg ticked: String,
    ): Pair<LibraryViewModel, List<DocumentIdModel>> {
        val documents = SampleLibrary.open(app = app, "a.txt", "b.txt", "c.txt", "archive/old.txt")
        val library = app.libraryViewModel()
        library.state.first { it.documents.size == documents.size }
        val ids = ticked.map { documents.getValue(key = it).id }
        ids.forEach { library.documentSelection.toggle(documentId = it) }
        library.state.first { it.selectedDocuments.size == ids.size }
        return library to ids
    }
}
