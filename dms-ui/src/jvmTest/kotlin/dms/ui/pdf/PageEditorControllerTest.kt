package dms.ui.pdf

import dms.application.port.storage.model.StorageResult
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.LibraryFolderModel
import dms.domain.library.model.RelativePathModel
import dms.ui.components.dialog.DialogController
import dms.ui.support.SamplePdfs
import dms.ui.support.TestApp
import dms.ui.support.UiTest
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.first
import java.io.ByteArrayInputStream
import java.nio.file.Files
import javax.imageio.ImageIO
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class PageEditorControllerTest {

    @Test
    fun `opening a PDF lists its pages and loads a thumbnail of each`() {
        UiTest.run { app ->
            val document = libraryWithPdf(app = app, pages = 3)
            val editor = editorOf(app = app, scope = backgroundScope)

            editor.open(document = document)

            val loaded = editor.state.first { it?.thumbnails?.size == 3 }
            assertEquals(expected = listOf(0, 1, 2), actual = loaded?.pages?.map { it.source })
            assertEquals(expected = false, actual = loaded?.loading)
            assertEquals(expected = 3, actual = app.platform.decodedImages)
        }
    }

    @Test
    fun `turned, removed and moved pages become a new version in every folder`() {
        UiTest.run { app ->
            val document = libraryWithPdf(app = app, pages = 3)
            val editor = editorOf(app = app, scope = backgroundScope)
            editor.open(document = document)
            editor.state.first { it?.loading == false }

            editor.rotate(position = 2, degrees = -90)
            editor.toggleDelete(position = 1)
            editor.move(position = 2, by = -2)
            editor.move(position = 0, by = -1)
            val edited = assertNotNull(actual = editor.state.value)
            assertEquals(expected = listOf(2, 0, 1), actual = edited.pages.map { it.source })
            assertEquals(expected = 270, actual = edited.pages.first().rotation)
            editor.apply()
            app.idle()

            assertNull(actual = editor.state.value)
            assertEquals(
                expected = "Saved the pages of sample-scan.pdf",
                actual = app.session.state.value.message?.text,
            )
            assertEquals(expected = 2, actual = pageCount(app = app, path = document.path))
            assertEquals(expected = 2, actual = pageCount(app = app, path = document.path, folder = syncOf(app = app)))
            val firstPage = assertNotNull(actual = renderedSize(app = app, path = document.path, page = 0))
            assertTrue(actual = firstPage.second > firstPage.first, message = "The turned page is upright")
        }
    }

    @Test
    fun `a split without trashing the original adds a document per part and keeps the original`() {
        UiTest.run { app ->
            val document = libraryWithPdf(app = app, pages = 3)
            val editor = editorOf(app = app, scope = backgroundScope)
            editor.open(document = document)
            editor.state.first { it?.loading == false }

            editor.toggleSplit(position = 0)
            editor.apply()
            app.idle()

            assertNull(actual = editor.state.value)
            assertEquals(
                expected = "Split sample-scan.pdf into 2 documents",
                actual = app.session.state.value.message?.text,
            )
            val parts = documents(app = app).filter { it.id != document.id }
            assertEquals(
                expected = listOf(1, 2),
                actual = parts.mapNotNull { pageCount(app = app, path = it.path) }.sorted(),
            )
            assertTrue(actual = Files.exists(app.primary.resolve("scans/sample-scan.pdf")))
            assertEquals(expected = 3, actual = documents(app = app).size)
        }
    }

    @Test
    fun `a split that trashes the original asks first and changes nothing before the answer`() {
        UiTest.run { app ->
            val document = libraryWithPdf(app = app, pages = 2)
            val dialogs = DialogController()
            val editor = editorOf(app = app, scope = backgroundScope, dialogs = dialogs)
            editor.open(document = document)
            editor.state.first { it?.loading == false }
            editor.toggleSplit(position = 0)
            editor.setTrashOriginal(trash = true)

            editor.apply()

            val question = assertNotNull(actual = dialogs.confirmation.value)
            assertEquals(expected = listOf("sample-scan.pdf"), actual = question.affected)
            assertEquals(expected = 1, actual = documents(app = app).size)
            assertTrue(actual = Files.exists(app.sync.resolve("scans/sample-scan.pdf")))
            assertNotNull(actual = editor.state.value)

            dialogs.confirm()
            app.idle()

            val remaining = documents(app = app)
            assertEquals(expected = 2, actual = remaining.size)
            assertTrue(actual = remaining.none { it.id == document.id })
            assertFalse(actual = Files.exists(app.primary.resolve("scans/sample-scan.pdf")))
            assertFalse(actual = Files.exists(app.sync.resolve("scans/sample-scan.pdf")))
        }
    }

    @Test
    fun `a split mark on the last page or on a removed page splits nothing`() {
        UiTest.run { app ->
            val document = libraryWithPdf(app = app, pages = 3)
            val editor = editorOf(app = app, scope = backgroundScope)
            editor.open(document = document)
            editor.state.first { it?.loading == false }

            editor.toggleSplit(position = 2)
            editor.toggleSplit(position = 1)
            editor.toggleDelete(position = 1)
            assertEquals(expected = 0, actual = editor.state.value?.splits)
            editor.apply()
            app.idle()

            assertEquals(expected = 1, actual = documents(app = app).size)
            assertEquals(expected = 2, actual = pageCount(app = app, path = document.path))
        }
    }

    @Test
    fun `removing every page writes nothing and keeps the editor open`() {
        UiTest.run { app ->
            val document = libraryWithPdf(app = app, pages = 2)
            val editor = editorOf(app = app, scope = backgroundScope)
            editor.open(document = document)
            editor.state.first { it?.loading == false }

            editor.toggleDelete(position = 0)
            editor.toggleDelete(position = 1)
            editor.apply()

            assertNotNull(actual = editor.state.value)
            assertTrue(actual = app.session.state.value.operations.isEmpty())
            assertEquals(expected = 2, actual = pageCount(app = app, path = document.path))
        }
    }

    @Test
    fun `while the library is busy the edit is not started and the editor stays open`() {
        UiTest.run { app ->
            val document = libraryWithPdf(app = app, pages = 2)
            val editor = editorOf(app = app, scope = backgroundScope)
            editor.open(document = document)
            editor.state.first { it?.loading == false }
            editor.rotate(position = 0, degrees = 90)
            val gate = CompletableDeferred<Unit>()
            app.session.perform(label = "Sample work") { gate.await() }

            editor.apply()
            gate.complete(value = Unit)
            app.idle()

            assertEquals(expected = 90, actual = editor.state.value?.pages?.first()?.rotation)
            val firstPage = assertNotNull(actual = renderedSize(app = app, path = document.path, page = 0))
            assertTrue(actual = firstPage.first > firstPage.second, message = "The page was not turned")
        }
    }

    @Test
    fun `a file that vanished is reported and nothing is split`() {
        UiTest.run { app ->
            val document = libraryWithPdf(app = app, pages = 2)
            val editor = editorOf(app = app, scope = backgroundScope)
            editor.open(document = document)
            editor.state.first { it?.loading == false }
            editor.rotate(position = 0, degrees = 90)
            editor.toggleSplit(position = 0)
            Files.delete(app.primary.resolve("scans/sample-scan.pdf"))

            editor.apply()
            app.idle()

            val message = assertNotNull(actual = app.session.state.value.message)
            assertTrue(actual = message.isError)
            assertEquals(expected = 1, actual = documents(app = app).size)
        }
    }

    private suspend fun libraryWithPdf(app: TestApp, pages: Int): DocumentModel {
        SamplePdfs.write(file = app.primary.resolve("scans/sample-scan.pdf"), pages = pages)
        app.configure(addNewFiles = true)
        app.session.syncToAllDestinations()
        app.idle()
        return documents(app = app).single()
    }

    private fun editorOf(
        app: TestApp,
        scope: CoroutineScope,
        dialogs: DialogController = DialogController(),
    ): PageEditorController {
        return PageEditorController(
            services = app.services,
            session = app.session,
            platform = app.platform,
            dialogs = dialogs,
            scope = scope,
        )
    }

    private fun documents(app: TestApp): List<DocumentModel> {
        return app.services.library.catalog.value?.documents?.values.orEmpty().toList()
    }

    private fun primaryOf(app: TestApp): LibraryFolderModel {
        return checkNotNull(value = app.session.state.value.folders).primary
    }

    private fun syncOf(app: TestApp): LibraryFolderModel {
        return checkNotNull(value = app.session.state.value.folders).syncFolders.single()
    }

    private suspend fun pageCount(
        app: TestApp,
        path: RelativePathModel,
        folder: LibraryFolderModel = primaryOf(app = app),
    ): Int? {
        val counted = app.services.pdf.editor.pageCount(folder = folder, path = path)
        return (counted as? StorageResult.Success)?.value
    }

    /** Width and height of [page] as the page editor renders it. */
    private suspend fun renderedSize(app: TestApp, path: RelativePathModel, page: Int): Pair<Int, Int>? {
        val image = app.services.pdf.editor.renderPage(
            folder = primaryOf(app = app),
            path = path,
            page = page,
            maxSide = 100,
        ) ?: return null
        val decoded = ImageIO.read(ByteArrayInputStream(image.bytes)) ?: return null
        return decoded.width to decoded.height
    }
}
