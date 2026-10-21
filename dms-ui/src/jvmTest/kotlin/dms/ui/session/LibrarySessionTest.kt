package dms.ui.session

import dms.ui.session.model.OperationLane
import dms.ui.support.UiTest
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.flow.first
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class LibrarySessionTest {

    @Test
    fun `with the preference on, new files join the library during the scan`() {
        UiTest.run { app ->
            app.write(file = app.primary.resolve("letters/a.pdf"), content = "alpha")
            app.write(file = app.sync.resolve("b.pdf"), content = "beta")

            app.configure(addNewFiles = true)

            val paths = app.services.library.catalog.first { it?.documents?.size == 2 }
                ?.documents?.values?.map { it.path.value }?.toSet()
            assertEquals(expected = setOf("letters/a.pdf", "b.pdf"), actual = paths)
            assertEquals(expected = "beta", actual = app.read(file = app.primary.resolve("b.pdf")))
            assertTrue(actual = app.session.progress.value.isEmpty())
            val log = app.session.log.entries.value.map { it.text }
            assertTrue(actual = "Read letters/a.pdf" in log)
            assertTrue(actual = "Adding new files: b.pdf" in log)
        }
    }

    @Test
    fun `with the preference off, a scan leaves new files to the dashboard`() {
        UiTest.run { app ->
            app.write(file = app.primary.resolve("a.pdf"), content = "alpha")

            app.configure(addNewFiles = false)

            assertEquals(expected = 0, actual = app.services.library.catalog.value?.documents?.size)
            assertEquals(expected = 1, actual = app.session.state.value.lastScan?.changes?.size)
        }
    }

    @Test
    fun `only a cancellable operation can be cancelled`() {
        UiTest.run { app ->
            app.configure()

            app.session.perform(label = "Waiting") { awaitCancellation() }
            app.session.cancel(lane = OperationLane.LIBRARY)
            assertEquals(expected = "Waiting", actual = app.session.state.value.busy)
            assertFalse(actual = app.session.state.value.operations.single().cancellable)
        }
    }

    @Test
    fun `an AI run and a library operation run side by side, but two of a lane do not`() {
        UiTest.run { app ->
            app.configure()

            assertTrue(actual = app.session.perform(label = "AI", lane = OperationLane.AI) { awaitCancellation() })
            assertTrue(actual = app.session.perform(label = "Library") { awaitCancellation() })
            assertFalse(actual = app.session.perform(label = "Second AI", lane = OperationLane.AI) {})
            assertFalse(actual = app.session.perform(label = "Second library") {})

            assertEquals(expected = "AI", actual = app.session.state.value.aiBusy)
            assertEquals(expected = "Library", actual = app.session.state.value.busy)
        }
    }

    @Test
    fun `a cancelled operation ends and says so`() {
        UiTest.run { app ->
            app.configure()

            app.session.perform(label = "Waiting", cancellable = true) { awaitCancellation() }
            app.session.cancel(lane = OperationLane.LIBRARY)
            app.idle()

            assertEquals(expected = "Waiting: cancelled", actual = app.session.state.value.message?.text)
            assertTrue(actual = app.session.log.entries.value.any { it.text == "Waiting: cancelled" })
        }
    }
}
