package dms.ui.pdf.rebuild

import dms.application.pdf.rebuild.model.RebuildChoice
import dms.domain.document.model.DocumentModel
import dms.ui.components.dialog.DialogController
import dms.ui.support.SamplePdfs
import dms.ui.support.TestApp
import dms.ui.support.UiTest
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.first
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class RebuildControllerTest {

    @Test
    fun `a converted scan opens the comparison with both pages, and using the rebuild asks first`() {
        UiTest.run { app ->
            val document = libraryWithScan(app = app)
            val dialogs = DialogController()
            val controller = controllerOf(app = app, scope = backgroundScope, dialogs = dialogs)

            controller.check(document = document)
            controller.available.first { document.id in it }
            controller.start(document = document)
            app.idle()
            val compare = controller.compare.first { it?.original != null && it.rebuiltImage != null }

            assertEquals(expected = 1, actual = compare?.pages)
            controller.choose(choice = RebuildChoice.REPLACE_WITH_REBUILD)
            val question = assertNotNull(actual = dialogs.confirmation.value)
            assertEquals(expected = listOf("scans/sample-scan.pdf"), actual = question.affected)
            assertEquals(expected = document.contentHash, actual = documentOf(app = app).contentHash)

            dialogs.confirm()
            app.idle()

            assertNull(actual = controller.compare.value)
            assertNotEquals(illegal = document.contentHash, actual = documentOf(app = app).contentHash)
            // The scan goes to the trash from the primary and from the sync folder.
            assertEquals(expected = 2, actual = Files.list(app.trash).use { it.count() })
        }
    }

    @Test
    fun `keeping both adds the rebuild next to the scan`() {
        UiTest.run { app ->
            val document = libraryWithScan(app = app)
            val controller = controllerOf(app = app, scope = backgroundScope)
            controller.start(document = document)
            app.idle()
            controller.compare.first { it != null }

            controller.choose(choice = RebuildChoice.KEEP_BOTH)
            app.idle()

            assertNull(actual = controller.compare.value)
            assertTrue(actual = Files.exists(app.primary.resolve("scans/sample-scan (rebuilt).pdf")))
            assertTrue(actual = Files.exists(app.primary.resolve("scans/sample-scan.pdf")))
        }
    }

    @Test
    fun `a second conversion waits while a comparison is open`() {
        UiTest.run { app ->
            val document = libraryWithScan(app = app)
            val controller = controllerOf(app = app, scope = backgroundScope)
            controller.start(document = document)
            app.idle()
            val first = controller.compare.first { it != null }
            val requests = app.ai.chatRequests

            controller.start(document = document)
            app.idle()

            assertEquals(expected = requests, actual = app.ai.chatRequests)
            assertEquals(expected = first?.rebuilt, actual = controller.compare.value?.rebuilt)
            assertEquals(
                expected = "Choose what to keep in the open comparison first",
                actual = app.session.state.value.message?.text,
            )
        }
    }

    @Test
    fun `an AI that cannot be reached says so and opens nothing`() {
        UiTest.run { app ->
            val document = libraryWithScan(app = app)
            val controller = controllerOf(app = app, scope = backgroundScope)
            app.ai.reachable = false

            controller.start(document = document)
            app.idle()

            val message = assertNotNull(actual = app.session.state.value.message)
            assertEquals(
                expected = "AI is not configured or cannot be reached: check Settings › AI",
                actual = message.text,
            )
            assertNull(actual = controller.compare.value)
        }
    }

    private suspend fun libraryWithScan(app: TestApp): DocumentModel {
        SamplePdfs.write(file = app.primary.resolve("scans/sample-scan.pdf"), pages = 1)
        app.ai.suggestion = LAYOUT
        app.configure(addNewFiles = true)
        app.session.syncToAllDestinations()
        app.idle()
        return documentOf(app = app)
    }

    private fun documentOf(app: TestApp): DocumentModel {
        return app.services.library.catalog.value?.documents?.values.orEmpty()
            .first { it.path.value == "scans/sample-scan.pdf" }
    }

    private fun controllerOf(
        app: TestApp,
        scope: CoroutineScope,
        dialogs: DialogController = DialogController(),
    ): RebuildController {
        return RebuildController(
            services = app.services,
            session = app.session,
            platform = app.platform,
            dialogs = dialogs,
            scope = scope,
        )
    }

    private companion object {
        /** The sample page's "Page 1", as a vision model would place it. */
        const val LAYOUT = """{"lines":[{"bbox":[100,410,290,500],"text":"Page 1","bold":false,""" +
            """"font":"sans","legible":true}],"graphics":[]}"""
    }
}
