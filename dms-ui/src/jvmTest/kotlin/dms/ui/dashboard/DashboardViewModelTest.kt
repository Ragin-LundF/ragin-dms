package dms.ui.dashboard

import dms.ui.dashboard.model.DashboardUiState
import dms.ui.dashboard.model.TaskGroup
import dms.ui.dashboard.model.TaskRowState
import dms.ui.support.TestApp
import dms.ui.support.UiTest
import kotlinx.coroutines.flow.first
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertTrue

class DashboardViewModelTest {

    @Test
    fun `a file added outside the tool is adopted into every folder`() {
        UiTest.run { app ->
            app.write(file = app.primary.resolve("letters/note-a.pdf"), content = "alpha")
            app.configure()
            val viewModel = DashboardViewModel(services = app.services, session = app.session)

            val row = rowOf(state = awaitGroup(viewModel = viewModel, group = TaskGroup.NEW_FILES))
            row.buttons.single { it.label == "Adopt" }.onClick()
            app.idle()

            val state = viewModel.state.first { state -> state.groups.none { it.group == TaskGroup.NEW_FILES } }
            assertEquals(expected = 1, actual = state.documentCount)
            assertEquals(expected = "alpha", actual = app.read(file = app.sync.resolve("letters/note-a.pdf")))
        }
    }

    @Test
    fun `adopt all adopts every new file`() {
        UiTest.run { app ->
            app.write(file = app.primary.resolve("a.pdf"), content = "alpha")
            app.write(file = app.sync.resolve("b.pdf"), content = "beta")
            app.configure()
            val viewModel = DashboardViewModel(services = app.services, session = app.session)

            val group = awaitGroup(viewModel = viewModel, group = TaskGroup.NEW_FILES)
                .groups.single { it.group == TaskGroup.NEW_FILES }
            assertNotNull(actual = group.bulkAction).onClick()
            app.idle()

            viewModel.state.first { it.documentCount == 2 }
            assertTrue(actual = Files.exists(app.primary.resolve("b.pdf")))
            assertTrue(actual = Files.exists(app.sync.resolve("a.pdf")))
        }
    }

    @Test
    fun `a file deleted outside the tool is restored from the folder that still has it`() {
        UiTest.run { app ->
            adoptOne(app = app)
            Files.delete(app.sync.resolve("a.pdf"))
            val viewModel = DashboardViewModel(services = app.services, session = app.session)
            app.session.scan()
            app.idle()

            val row = rowOf(state = awaitGroup(viewModel = viewModel, group = TaskGroup.MISSING))
            row.buttons.single { it.label == "Restore" }.onClick()
            app.idle()

            viewModel.state.first { state -> state.groups.none { it.group == TaskGroup.MISSING } }
            assertEquals(expected = "alpha", actual = app.read(file = app.sync.resolve("a.pdf")))
        }
    }

    @Test
    fun `delete everywhere asks first, then moves the file to the trash`() {
        UiTest.run { app ->
            adoptOne(app = app)
            Files.delete(app.sync.resolve("a.pdf"))
            val viewModel = DashboardViewModel(services = app.services, session = app.session)
            app.session.scan()
            app.idle()

            val row = rowOf(state = awaitGroup(viewModel = viewModel, group = TaskGroup.MISSING))
            row.buttons.single { it.label.startsWith(prefix = "Delete everywhere") }.onClick()

            assertNotNull(actual = viewModel.dialogs.confirmation.value)
            assertTrue(actual = Files.exists(app.primary.resolve("a.pdf")))

            viewModel.dialogs.confirm()
            app.idle()

            viewModel.state.first { it.documentCount == 0 }
            assertFalse(actual = Files.exists(app.primary.resolve("a.pdf")))
            assertEquals(expected = 1, actual = Files.list(app.trash).use { it.count() })
        }
    }

    @Test
    fun `a dismissed task stays hidden after the next scan`() {
        UiTest.run { app ->
            app.write(file = app.primary.resolve("a.pdf"), content = "alpha")
            app.configure()
            val viewModel = DashboardViewModel(services = app.services, session = app.session)

            val row = rowOf(state = awaitGroup(viewModel = viewModel, group = TaskGroup.NEW_FILES))
            row.buttons.single { it.label == "Dismiss" }.onClick()
            app.idle()
            app.session.scan()
            app.idle()

            val state = viewModel.state.first { it.groups.isEmpty() && it.lastScanAt != null }
            assertEquals(expected = 0, actual = state.documentCount)
        }
    }

    private suspend fun adoptOne(app: TestApp) {
        app.write(file = app.primary.resolve("a.pdf"), content = "alpha")
        app.configure()
        val viewModel = DashboardViewModel(services = app.services, session = app.session)
        rowOf(state = awaitGroup(viewModel = viewModel, group = TaskGroup.NEW_FILES))
            .buttons.single { it.label == "Adopt" }.onClick()
        app.idle()
    }

    private suspend fun awaitGroup(viewModel: DashboardViewModel, group: TaskGroup): DashboardUiState {
        return viewModel.state.first { state -> state.groups.any { it.group == group } }
    }

    private fun rowOf(state: DashboardUiState): TaskRowState {
        return state.groups.single().rows.single()
    }
}
