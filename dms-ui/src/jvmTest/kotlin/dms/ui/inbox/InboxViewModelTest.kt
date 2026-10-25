package dms.ui.inbox

import dms.domain.document.model.DocumentIdModel
import dms.domain.settings.model.InboxSettingsModel
import dms.ui.inbox.model.InboxRowModel
import dms.ui.support.SampleImages
import dms.ui.support.TestApp
import dms.ui.support.UiTest
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.flow.first
import java.awt.image.BufferedImage
import java.nio.file.Files
import java.nio.file.Path
import javax.imageio.ImageIO
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class InboxViewModelTest {

    private val answer = """
        {"title":"Sample bill","documentDate":"2024-03-15","correspondent":"Sample Energy","tags":["Tax"]}
    """.trimIndent()

    @Test
    fun `documents dropped into the inbox are listed, and the inbox is not offered as a target`() {
        UiTest.run { app ->
            app.write(file = app.primary.resolve("letters/old-letter.txt"), content = "old")
            libraryWithInbox(app = app)
            val inbox = InboxViewModel(services = app.services, session = app.session)

            SampleImages.write(file = app.primary.resolve("Inbox/scan-1.png"))
            app.write(file = app.primary.resolve("Inbox/post/letter-2.txt"), content = "beta")
            app.session.scan()
            app.idle()

            val state = inbox.state.first { it.rows.size == 2 }
            assertTrue(actual = state.enabled)
            assertEquals(expected = "Inbox", actual = state.path)
            val scan = rowNamed(rows = state.rows, name = "scan-1.png")
            val letter = rowNamed(rows = state.rows, name = "letter-2.txt")
            assertEquals(expected = "" to true, actual = scan.subfolder to scan.analyzable)
            assertEquals(expected = "post" to false, actual = letter.subfolder to letter.analyzable)
            assertEquals(expected = listOf("letters"), actual = state.folders)
            assertTrue(actual = Files.exists(app.sync.resolve("Inbox/post/letter-2.txt")))
        }
    }

    @Test
    fun `while the inbox is off its folder holds ordinary documents and nothing is listed`() {
        UiTest.run { app ->
            app.write(file = app.primary.resolve("Inbox/letter-2.txt"), content = "beta")
            app.configure(addNewFiles = true)
            val inbox = InboxViewModel(services = app.services, session = app.session)

            inbox.startMove(documentId = DocumentIdModel(value = "unknown-document"))

            val state = inbox.state.value
            assertFalse(actual = state.enabled)
            assertEquals(expected = emptyList(), actual = state.rows)
            assertNull(actual = state.pick)
            assertEquals(expected = 1, actual = app.services.library.catalog.value?.documents?.size)
        }
    }

    @Test
    fun `filing moves the document out of the inbox in every folder and remembers the folder`() {
        UiTest.run { app ->
            libraryWithInbox(app = app)
            val inbox = InboxViewModel(services = app.services, session = app.session)
            arrive(app = app, names = listOf("letter-1.txt", "letter-2.txt"))
            val rows = inbox.state.first { it.rows.size == 2 }.rows
            val first = rowNamed(rows = rows, name = "letter-1.txt")

            inbox.startMove(documentId = first.documentId)
            assertEquals(expected = "", actual = inbox.state.value.pick?.folder)
            assertEquals(expected = "letter-1.txt", actual = inbox.state.value.pick?.fileName)
            inbox.updatePick(folder = " letters / 2024 ", fileName = " renamed.txt ")
            inbox.confirmMove()
            app.idle()

            assertNull(actual = inbox.state.value.pick)
            assertEquals(
                expected = "Filed letter-1.txt in letters/2024",
                actual = app.session.state.value.message?.text,
            )
            assertTrue(actual = Files.exists(app.primary.resolve("letters/2024/renamed.txt")))
            assertTrue(actual = Files.exists(app.sync.resolve("letters/2024/renamed.txt")))
            assertFalse(actual = Files.exists(app.primary.resolve("Inbox/letter-1.txt")))
            val state = inbox.state.first { it.rows.size == 1 }
            assertEquals(expected = listOf("letters/2024"), actual = state.recentTargets)

            inbox.startMove(documentId = state.rows.single().documentId)
            assertEquals(expected = "letters/2024", actual = inbox.state.value.pick?.folder)
        }
    }

    @Test
    fun `the saved path template suggests the folder to file into`() {
        UiTest.run { app ->
            libraryWithInbox(app = app)
            app.services.documents.changeSettings(subject = "Path template") { settings ->
                settings.withLayout(layout = settings.layout.copy(pathTemplate = "Archive/{kind}"))
            }
            val inbox = InboxViewModel(services = app.services, session = app.session)
            arrive(app = app, names = listOf("letter-1.txt"))

            val row = inbox.state.first { it.rows.isNotEmpty() }.rows.single()
            inbox.startMove(documentId = row.documentId)

            assertEquals(expected = "Archive/Other", actual = row.prefill)
            assertEquals(expected = "Archive/Other", actual = inbox.state.value.pick?.folder)
        }
    }

    @Test
    fun `filing into the inbox itself is refused and the document stays`() {
        UiTest.run { app ->
            libraryWithInbox(app = app)
            val inbox = InboxViewModel(services = app.services, session = app.session)
            arrive(app = app, names = listOf("letter-1.txt"))
            val row = inbox.state.first { it.rows.isNotEmpty() }.rows.single()

            inbox.startMove(documentId = row.documentId)
            inbox.updatePick(folder = "Inbox/later", fileName = row.name)
            inbox.confirmMove()
            app.idle()

            val message = assertNotNull(actual = app.session.state.value.message)
            assertTrue(actual = message.isError)
            assertTrue(actual = Files.exists(app.primary.resolve("Inbox/letter-1.txt")))
            assertEquals(expected = emptyList(), actual = inbox.state.value.recentTargets)
            assertEquals(expected = 1, actual = inbox.state.value.rows.size)
        }
    }

    @Test
    fun `while the library is busy filing is not started and the picker stays open`() {
        UiTest.run { app ->
            libraryWithInbox(app = app)
            val inbox = InboxViewModel(services = app.services, session = app.session)
            arrive(app = app, names = listOf("letter-1.txt"))
            val row = inbox.state.first { it.rows.isNotEmpty() }.rows.single()
            val gate = CompletableDeferred<Unit>()
            app.session.perform(label = "Sample work") { gate.await() }

            inbox.startMove(documentId = row.documentId)
            inbox.updatePick(folder = "letters", fileName = row.name)
            inbox.confirmMove()
            gate.complete(value = Unit)
            app.idle()

            assertEquals(expected = "letters", actual = inbox.state.value.pick?.folder)
            assertTrue(actual = Files.exists(app.primary.resolve("Inbox/letter-1.txt")))
        }
    }

    @Test
    fun `an analyzed document shows the AI's suggestion, which opens a review`() {
        UiTest.run { app ->
            libraryWithInbox(app = app)
            val inbox = InboxViewModel(services = app.services, session = app.session)
            SampleImages.write(file = app.primary.resolve("Inbox/scan-1.png"))
            app.session.scan()
            app.idle()
            val row = inbox.state.first { it.rows.isNotEmpty() }.rows.single()
            app.ai.suggestion = answer

            inbox.analyze(documentId = row.documentId)
            app.idle()

            val suggestion = assertNotNull(
                actual = inbox.state.first { state -> state.rows.single().suggestion != null }.rows.single().suggestion,
            )
            assertEquals(expected = "Sample bill", actual = suggestion.title)
            assertEquals(
                expected = "The AI suggested a classification for scan-1.png",
                actual = app.session.state.value.message?.text,
            )
            inbox.review(suggestion = suggestion)
            assertEquals(expected = "Sample bill", actual = inbox.review.state.value?.draft?.title?.value)
            assertTrue(actual = Files.exists(app.primary.resolve("Inbox/scan-1.png")))
        }
    }

    @Test
    fun `a stopped AI server is reported and no suggestion appears`() {
        UiTest.run { app ->
            libraryWithInbox(app = app)
            val inbox = InboxViewModel(services = app.services, session = app.session)
            SampleImages.write(file = app.primary.resolve("Inbox/scan-1.png"))
            app.session.scan()
            app.idle()
            val row = inbox.state.first { it.rows.isNotEmpty() }.rows.single()
            app.ai.reachable = false

            inbox.analyze(documentId = row.documentId)
            app.idle()

            val message = assertNotNull(actual = app.session.state.value.message)
            assertTrue(actual = message.isError)
            assertTrue(actual = "does not answer" in message.text)
            assertNull(actual = inbox.state.value.rows.single().suggestion)
        }
    }

    @Test
    fun `analyze all asks the AI about every inbox document and nothing outside it`() {
        UiTest.run { app ->
            writeImage(file = app.primary.resolve("photos/outside.png"), width = 4)
            libraryWithInbox(app = app)
            val inbox = InboxViewModel(services = app.services, session = app.session)
            writeImage(file = app.primary.resolve("Inbox/scan-1.png"), width = 5)
            writeImage(file = app.primary.resolve("Inbox/scan-2.png"), width = 6)
            app.session.scan()
            app.idle()
            inbox.state.first { it.rows.size == 2 }
            app.ai.suggestion = answer

            inbox.analyzeAll()
            app.idle()

            inbox.state.first { state -> state.rows.all { it.suggestion != null } }
            assertEquals(expected = 2, actual = app.ai.chatRequests)
        }
    }

    /**
     * Opens the library with every file already there catalogued, then leaves new files to the inbox
     * (and the dashboard) and switches the inbox on.
     */
    private suspend fun libraryWithInbox(app: TestApp) {
        app.configure(addNewFiles = true)
        app.session.setAddNewFilesWhileScanning(enabled = false)
        app.idle()
        app.services.organize.configureInbox(inbox = InboxSettingsModel(enabled = true))
    }

    /** Drops text files into the inbox of the primary and scans, as the folder watcher would. */
    private suspend fun arrive(app: TestApp, names: List<String>) {
        names.forEach { app.write(file = app.primary.resolve("Inbox/$it"), content = "content of $it") }
        app.session.scan()
        app.idle()
    }

    /** A PNG whose content differs by [width], so the library does not take it for a copy. */
    private fun writeImage(file: Path, width: Int) {
        Files.createDirectories(file.parent)
        ImageIO.write(BufferedImage(width, 1, BufferedImage.TYPE_INT_RGB), "png", file.toFile())
    }

    private fun rowNamed(rows: List<InboxRowModel>, name: String): InboxRowModel {
        return rows.single { it.name == name }
    }
}
