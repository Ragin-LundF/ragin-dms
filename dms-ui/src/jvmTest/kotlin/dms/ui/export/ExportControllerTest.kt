package dms.ui.export

import dms.application.export.model.ExportOptionsModel
import dms.application.export.model.ExportScope
import dms.domain.library.model.RelativePathModel
import dms.domain.query.model.DocumentQueryModel
import dms.domain.query.model.SavedSearchIdModel
import dms.domain.query.model.SavedSearchModel
import dms.ui.support.TestApp
import dms.ui.support.UiTest
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.runCurrent
import java.nio.file.Files
import java.nio.file.Path
import java.time.LocalDate
import kotlin.io.path.isRegularFile
import kotlin.io.path.name
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class ExportControllerTest {

    private val letters = DocumentQueryModel(folder = RelativePathModel.of(raw = "letters"))

    @Test
    fun `the current view is exported into the chosen folder outside the library`() {
        UiTest.run { app ->
            libraryWithTwoFiles(app = app)
            val export = controllerOf(app = app, scope = backgroundScope, currentQuery = { letters })
            val target = Files.createDirectories(app.root.resolve("outside"))
            app.platform.nextFolder = "$target"

            export.openCurrentView(title = "Letters")
            export.export()
            runCurrent()
            app.idle()

            assertNull(actual = export.draft.value)
            val message = assertNotNull(actual = app.session.state.value.message)
            assertFalse(actual = message.isError)
            assertTrue(actual = message.text.startsWith(prefix = "Exported 1 documents to "))
            assertTrue(actual = "letter-b.txt" in fileNamesIn(directory = target))
            assertFalse(actual = "note-a.txt" in fileNamesIn(directory = target))
        }
    }

    @Test
    fun `a smart folder is exported as a ZIP file under the name typed`() {
        UiTest.run { app ->
            libraryWithTwoFiles(app = app)
            val export = controllerOf(app = app, scope = backgroundScope)
            val target = Files.createDirectories(app.root.resolve("outside"))
            app.platform.nextFolder = "$target"
            val search = SavedSearchModel(id = SavedSearchIdModel(value = "s-1"), name = "Letters", query = letters)

            export.openSmartFolder(search = search)
            export.setName(name = "  Sample export ")
            export.setOptions(options = ExportOptionsModel(zip = true))
            export.export()
            runCurrent()
            app.idle()

            assertEquals(expected = listOf("Sample export.zip"), actual = fileNamesIn(directory = target))
        }
    }

    @Test
    fun `a folder inside the library is refused`() {
        UiTest.run { app ->
            libraryWithTwoFiles(app = app)
            val export = controllerOf(app = app, scope = backgroundScope)
            app.platform.nextFolder = "${Files.createDirectories(app.primary.resolve("exports"))}"

            export.openCurrentView(title = "Everything")
            export.export()
            runCurrent()
            app.idle()

            val message = assertNotNull(actual = app.session.state.value.message)
            assertTrue(actual = message.isError)
            assertEquals(expected = "Choose a folder outside the library", actual = message.text)
            assertEquals(expected = emptyList(), actual = fileNamesIn(directory = app.primary.resolve("exports")))
        }
    }

    @Test
    fun `a correspondent without documents exports nothing and says so`() {
        UiTest.run { app ->
            libraryWithTwoFiles(app = app)
            val export = controllerOf(app = app, scope = backgroundScope)
            val target = Files.createDirectories(app.root.resolve("outside"))
            app.platform.nextFolder = "$target"

            export.openCurrentView(title = "Everything")
            export.setCorrespondent(name = " Sample Utility ")
            export.presetCorrespondent()
            val draft = assertNotNull(actual = export.draft.value)
            assertEquals(expected = "Everything from Sample Utility", actual = draft.scopeLabel)
            assertEquals(
                expected = ExportScope.Query(query = DocumentQueryModel(correspondent = "Sample Utility")),
                actual = draft.scope,
            )
            export.export()
            runCurrent()
            app.idle()

            val message = assertNotNull(actual = app.session.state.value.message)
            assertEquals(expected = "No document matches; nothing was exported", actual = message.text)
            assertTrue(actual = message.isError)
            assertEquals(expected = emptyList(), actual = fileNamesIn(directory = target))
        }
    }

    @Test
    fun `the tax year preset names the year and asks for the tax tag and its dates`() {
        UiTest.run { app ->
            val export = controllerOf(app = app, scope = backgroundScope)

            export.presetTaxYear(year = 2024)
            assertNull(actual = export.draft.value)
            export.openCurrentView(title = "Everything")
            export.presetTaxYear(year = 2024)

            val draft = assertNotNull(actual = export.draft.value)
            assertEquals(expected = "Tax year 2024", actual = draft.scopeLabel)
            assertEquals(expected = "Tax 2024", actual = draft.name)
            val query = (draft.scope as ExportScope.Query).query
            assertEquals(expected = listOf("Tax"), actual = query.tags.all.map { it.value })
            assertEquals(expected = LocalDate.of(2024, 1, 1), actual = query.dates?.from)
            assertEquals(expected = LocalDate.of(2024, 12, 31), actual = query.dates?.to)
        }
    }

    @Test
    fun `a cancelled folder choice or a blank name keeps the draft and exports nothing`() {
        UiTest.run { app ->
            libraryWithTwoFiles(app = app)
            val export = controllerOf(app = app, scope = backgroundScope)
            export.openCurrentView(title = "Everything")
            app.platform.nextFolder = null

            export.export()
            runCurrent()
            app.idle()
            assertNotNull(actual = export.draft.value)

            app.platform.nextFolder = "${Files.createDirectories(app.root.resolve("outside"))}"
            export.setName(name = "   ")
            export.export()
            runCurrent()
            app.idle()

            assertNotNull(actual = export.draft.value)
            assertNull(actual = app.session.state.value.message)
            assertEquals(expected = emptyList(), actual = fileNamesIn(directory = app.root.resolve("outside")))
        }
    }

    @Test
    fun `while the library is busy the export is not started and the draft stays`() {
        UiTest.run { app ->
            libraryWithTwoFiles(app = app)
            val export = controllerOf(app = app, scope = backgroundScope)
            val target = Files.createDirectories(app.root.resolve("outside"))
            app.platform.nextFolder = "$target"
            val gate = CompletableDeferred<Unit>()
            app.session.perform(label = "Sample work") { gate.await() }

            export.openCurrentView(title = "Everything")
            export.export()
            runCurrent()
            gate.complete(value = Unit)
            app.idle()

            assertNotNull(actual = export.draft.value)
            assertEquals(expected = emptyList(), actual = fileNamesIn(directory = target))
        }
    }

    @Test
    fun `without an open library the export is refused`() {
        UiTest.run { app ->
            val export = controllerOf(app = app, scope = backgroundScope)
            app.platform.nextFolder = "${Files.createDirectories(app.root.resolve("outside"))}"

            export.openCurrentView(title = "Everything")
            export.export()
            runCurrent()
            app.idle()

            val message = assertNotNull(actual = app.session.state.value.message)
            assertEquals(expected = "No library is open", actual = message.text)
            assertTrue(actual = message.isError)
        }
    }

    private suspend fun libraryWithTwoFiles(app: TestApp) {
        app.write(file = app.primary.resolve("note-a.txt"), content = "alpha")
        app.write(file = app.primary.resolve("letters/letter-b.txt"), content = "beta")
        app.configure(addNewFiles = true)
    }

    private fun controllerOf(
        app: TestApp,
        scope: CoroutineScope,
        currentQuery: () -> DocumentQueryModel = { DocumentQueryModel.ALL },
    ): ExportController {
        return ExportController(
            services = app.services,
            session = app.session,
            platform = app.platform,
            scope = scope,
            currentQuery = currentQuery,
        )
    }

    private fun fileNamesIn(directory: Path): List<String> {
        if (!Files.exists(directory)) {
            return emptyList()
        }
        return Files.walk(directory).use { paths ->
            paths.filter { it.isRegularFile() }.map { it.name }.sorted().toList()
        }
    }
}
