package dms.ui.restructure

import dms.domain.library.model.RelativePathModel
import dms.ui.support.SampleImages
import dms.ui.support.TestApp
import dms.ui.support.UiTest
import kotlinx.coroutines.flow.first
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class RestructureViewModelTest {

    @Test
    fun `the preview groups the moves by target folder and changes nothing`() {
        UiTest.run { app ->
            libraryWithThreeFiles(app = app)
            val restructure = RestructureViewModel(services = app.services, session = app.session)

            restructure.setTemplate(text = "{kind}")
            restructure.preview()

            val state = restructure.state.first { it.planned }
            assertEquals(
                expected = listOf("Image" to 1, "Other" to 2),
                actual = state.groups.map { it.folder to it.count },
            )
            assertEquals(
                expected = listOf("a-note.txt → a-note.txt", "letters/b-letter.txt → b-letter.txt"),
                actual = state.groups.single { it.folder == "Other" }.samples,
            )
            assertEquals(expected = 3, actual = state.includedMoves)
            assertNull(actual = state.problem)
            assertTrue(actual = Files.exists(app.primary.resolve("letters/b-letter.txt")))
        }
    }

    @Test
    fun `documents without the value the template needs are skipped, and documents in place stay`() {
        UiTest.run { app ->
            app.write(file = app.primary.resolve("Other/kept.txt"), content = "kept")
            libraryWithThreeFiles(app = app)
            val restructure = RestructureViewModel(services = app.services, session = app.session)

            restructure.setTemplate(text = "{correspondent}")
            restructure.preview()
            val skipped = restructure.state.first { it.planned }
            restructure.setTemplate(text = "{kind}")
            restructure.preview()
            val planned = restructure.state.first { it.planned }

            assertEquals(expected = listOf("4 without correspondent"), actual = skipped.skipped)
            assertEquals(expected = emptyList(), actual = skipped.groups)
            assertEquals(expected = 1, actual = planned.unchanged)
            assertEquals(expected = 3, actual = planned.includedMoves)
        }
    }

    @Test
    fun `an invalid template shows the problem instead of a plan`() {
        UiTest.run { app ->
            libraryWithThreeFiles(app = app)
            val restructure = RestructureViewModel(services = app.services, session = app.session)

            restructure.setTemplate(text = "{kind")
            restructure.preview()
            val unclosed = restructure.state.first { it.problem != null }
            restructure.setTemplate(text = "{unknown}")
            restructure.preview()
            val unknown = restructure.state.first { it.problem != null }

            assertEquals(expected = "A { has no closing }", actual = unclosed.problem)
            assertEquals(expected = "Unknown placeholder; use the chips below", actual = unknown.problem)
            assertFalse(actual = unknown.planned)
        }
    }

    @Test
    fun `without an open library only the whole library is offered and the preview says why it fails`() {
        UiTest.run { app ->
            val restructure = RestructureViewModel(services = app.services, session = app.session)

            restructure.preview()

            val state = restructure.state.first { it.problem != null }
            assertEquals(expected = listOf("Whole library"), actual = state.scopes.map { it.label })
            assertEquals(expected = "No library is open", actual = state.problem)
            assertEquals(expected = "{category}/{year|Undated}", actual = state.template)
        }
    }

    @Test
    fun `applying asks first, moves only the ticked groups everywhere and offers to remove emptied folders`() {
        UiTest.run { app ->
            libraryWithThreeFiles(app = app)
            val restructure = RestructureViewModel(services = app.services, session = app.session)
            restructure.setTemplate(text = "{kind}")
            restructure.preview()
            restructure.state.first { it.planned }

            restructure.toggleGroup(folder = "Image")
            restructure.apply()
            val question = assertNotNull(actual = restructure.dialogs.confirmation.value)
            assertEquals(expected = "Move 2 documents", actual = question.title)
            assertTrue(actual = Files.exists(app.primary.resolve("a-note.txt")))
            restructure.dialogs.confirm()
            app.idle()

            assertEquals(expected = "Moved 2 documents", actual = app.session.state.value.message?.text)
            listOf(app.primary, app.sync).forEach { folder ->
                assertTrue(actual = Files.exists(folder.resolve("Other/a-note.txt")))
                assertTrue(actual = Files.exists(folder.resolve("Other/b-letter.txt")))
                assertTrue(actual = Files.exists(folder.resolve("photos/c-photo.png")))
            }
            val state = restructure.state.first { it.history.isNotEmpty() }
            assertFalse(actual = state.planned)
            val removal = assertNotNull(actual = restructure.dialogs.confirmation.value)
            assertEquals(expected = listOf("letters"), actual = removal.affected)

            restructure.dialogs.confirm()
            app.idle()

            assertFalse(actual = Files.exists(app.primary.resolve("letters")))
            assertFalse(actual = Files.exists(app.sync.resolve("letters")))
        }
    }

    @Test
    fun `an applied restructure can be reversed`() {
        UiTest.run { app ->
            libraryWithThreeFiles(app = app)
            val restructure = RestructureViewModel(services = app.services, session = app.session)
            restructure.setTemplate(text = "{kind}")
            restructure.preview()
            restructure.state.first { it.planned }
            restructure.apply()
            restructure.dialogs.confirm()
            app.idle()
            val name = restructure.state.first { it.history.isNotEmpty() }.history.single()
            restructure.dialogs.dismissConfirmation()

            restructure.reverse(name = name)
            assertEquals(expected = listOf(name), actual = restructure.dialogs.confirmation.value?.affected)
            restructure.dialogs.confirm()
            app.idle()

            listOf(app.primary, app.sync).forEach { folder ->
                assertTrue(actual = Files.exists(folder.resolve("a-note.txt")))
                assertTrue(actual = Files.exists(folder.resolve("letters/b-letter.txt")))
                assertTrue(actual = Files.exists(folder.resolve("photos/c-photo.png")))
            }
        }
    }

    @Test
    fun `nothing is asked or moved when every group is unticked`() {
        UiTest.run { app ->
            libraryWithThreeFiles(app = app)
            val restructure = RestructureViewModel(services = app.services, session = app.session)
            restructure.setTemplate(text = "{kind}")
            restructure.preview()
            restructure.state.first { it.planned }

            restructure.toggleGroup(folder = "Image")
            restructure.toggleGroup(folder = "Other")
            restructure.apply()

            assertEquals(expected = 0, actual = restructure.state.value.includedMoves)
            assertNull(actual = restructure.dialogs.confirmation.value)
            assertTrue(actual = app.session.state.value.operations.isEmpty())
        }
    }

    @Test
    fun `starting on a folder previews only that folder`() {
        UiTest.run { app ->
            libraryWithThreeFiles(app = app)
            val restructure = RestructureViewModel(services = app.services, session = app.session)
            restructure.setTemplate(text = "{kind}")

            restructure.start(folder = RelativePathModel.of(raw = "letters"))

            val state = restructure.state.first { it.planned }
            assertEquals(expected = "letters", actual = state.scopes[state.scopeIndex].label)
            assertEquals(expected = listOf("Other" to 1), actual = state.groups.map { it.folder to it.count })
        }
    }

    @Test
    fun `a saved template becomes the library's default`() {
        UiTest.run { app ->
            libraryWithThreeFiles(app = app)
            val restructure = RestructureViewModel(services = app.services, session = app.session)

            restructure.setTemplate(text = " {kind} ")
            restructure.saveTemplate()
            app.idle()

            assertEquals(expected = "{kind}", actual = restructure.state.value.savedTemplate)
            assertEquals(
                expected = "{kind}",
                actual = app.services.library.catalog.value?.settings?.layout?.pathTemplate,
            )
            val reopened = RestructureViewModel(services = app.services, session = app.session)
            assertEquals(expected = "{kind}", actual = reopened.state.value.template)
        }
    }

    @Test
    fun `a placeholder is added as a folder of its own`() {
        UiTest.run { app ->
            val restructure = RestructureViewModel(services = app.services, session = app.session)

            restructure.setTemplate(text = "")
            restructure.insertPlaceholder(token = "kind")
            val first = restructure.state.value.template
            restructure.setTemplate(text = "Archive/")
            restructure.insertPlaceholder(token = "year")

            assertEquals(expected = "{kind}", actual = first)
            assertEquals(expected = "Archive/{year}", actual = restructure.state.value.template)
        }
    }

    private suspend fun libraryWithThreeFiles(app: TestApp) {
        app.write(file = app.primary.resolve("a-note.txt"), content = "alpha")
        app.write(file = app.primary.resolve("letters/b-letter.txt"), content = "beta")
        SampleImages.write(file = app.primary.resolve("photos/c-photo.png"))
        app.configure(addNewFiles = true)
        app.session.syncToAllDestinations()
        app.idle()
    }
}
