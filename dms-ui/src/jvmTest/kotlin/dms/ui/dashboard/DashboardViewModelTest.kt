package dms.ui.dashboard

import dms.domain.change.DetectedChange
import dms.ui.dashboard.model.DashboardUiState
import dms.ui.dashboard.model.TaskGroup
import dms.ui.dashboard.model.TaskRowState
import dms.ui.support.TestApp
import dms.ui.support.UiTest
import kotlinx.coroutines.flow.first
import java.awt.image.BufferedImage
import java.nio.file.Files
import javax.imageio.ImageIO
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
            val viewModel = DashboardViewModel(services = app.services, session = app.session, platform = app.platform)

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
            val viewModel = DashboardViewModel(services = app.services, session = app.session, platform = app.platform)

            val group = awaitGroup(viewModel = viewModel, group = TaskGroup.NEW_FILES)
                .groups.single { it.group == TaskGroup.NEW_FILES }
            group.bulkActions.single().onClick()
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
            val viewModel = DashboardViewModel(services = app.services, session = app.session, platform = app.platform)
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
            val viewModel = DashboardViewModel(services = app.services, session = app.session, platform = app.platform)
            app.session.scan()
            app.idle()

            val row = rowOf(state = awaitGroup(viewModel = viewModel, group = TaskGroup.MISSING))
            row.buttons.single { it.label == "Move to trash…" }.onClick()

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
    fun `a file found on one side only can go to the trash after confirming`() {
        UiTest.run { app ->
            app.write(file = app.sync.resolve("stray.pdf"), content = "stray")
            app.configure()
            val viewModel = DashboardViewModel(services = app.services, session = app.session, platform = app.platform)

            val row = rowOf(state = awaitGroup(viewModel = viewModel, group = TaskGroup.NEW_FILES))
            assertEquals(expected = "Adopt", actual = row.buttons.first().label)
            row.buttons.single { it.label == "Move to trash…" }.onClick()
            assertTrue(actual = Files.exists(app.sync.resolve("stray.pdf")))
            viewModel.dialogs.confirm()
            app.idle()

            viewModel.state.first { state -> state.groups.none { it.group == TaskGroup.NEW_FILES } }
            assertFalse(actual = Files.exists(app.sync.resolve("stray.pdf")))
        }
    }

    @Test
    fun `a task opens its file from the folder that has it`() {
        UiTest.run { app ->
            app.write(file = app.sync.resolve("letters/stray.pdf"), content = "stray")
            app.configure()
            val viewModel = DashboardViewModel(services = app.services, session = app.session, platform = app.platform)

            val row = rowOf(state = awaitGroup(viewModel = viewModel, group = TaskGroup.NEW_FILES))
            row.opens.single().onClick()

            val expected = listOf(app.sync.resolve("letters/stray.pdf").toString())
            assertEquals(expected = expected, actual = app.platform.shown)
        }
    }

    @Test
    fun `a task shows a thumbnail of its image`() {
        UiTest.run { app ->
            Files.createDirectories(app.sync.resolve("photos"))
            val photo = BufferedImage(400, 300, BufferedImage.TYPE_INT_RGB)
            ImageIO.write(photo, "png", app.sync.resolve("photos/p.png").toFile())
            app.configure()
            val viewModel = DashboardViewModel(services = app.services, session = app.session, platform = app.platform)

            val row = rowOf(state = awaitGroup(viewModel = viewModel, group = TaskGroup.NEW_FILES))
            val preview = assertNotNull(actual = row.preview)
            val thumbnail = viewModel.thumbnails.load(folderId = preview.folderId, path = preview.path)

            assertNotNull(actual = thumbnail)
            assertEquals(expected = 1, actual = app.platform.decodedImages)
        }
    }

    @Test
    fun `a conflict can open the version of every folder`() {
        UiTest.run { app ->
            app.write(file = app.primary.resolve("a.pdf"), content = "alpha")
            app.write(file = app.sync.resolve("a.pdf"), content = "beta")
            app.configure()
            val viewModel = DashboardViewModel(services = app.services, session = app.session, platform = app.platform)

            val row = rowOf(state = awaitGroup(viewModel = viewModel, group = TaskGroup.CONFLICTS))
            row.opens.forEach { it.onClick() }

            val expected = listOf(app.primary.resolve("a.pdf").toString(), app.sync.resolve("a.pdf").toString())
            assertEquals(expected = expected, actual = app.platform.shown)
        }
    }

    @Test
    fun `a dismissed task stays hidden after the next scan`() {
        UiTest.run { app ->
            app.write(file = app.primary.resolve("a.pdf"), content = "alpha")
            app.configure()
            val viewModel = DashboardViewModel(services = app.services, session = app.session, platform = app.platform)

            val row = rowOf(state = awaitGroup(viewModel = viewModel, group = TaskGroup.NEW_FILES))
            row.buttons.single { it.label == "Dismiss" }.onClick()
            app.idle()
            app.session.scan()
            app.idle()

            val state = viewModel.state.first { it.groups.isEmpty() && it.lastScanAt != null }
            assertEquals(expected = 0, actual = state.documentCount)
        }
    }

    @Test
    fun `a file with another name in a sync folder is renamed there, not copied`() {
        UiTest.run { app ->
            adoptOne(app = app)
            Files.createDirectories(app.sync.resolve("older"))
            Files.move(app.sync.resolve("a.pdf"), app.sync.resolve("older/renamed.pdf"))
            val viewModel = DashboardViewModel(services = app.services, session = app.session, platform = app.platform)
            app.session.scan()
            app.idle()

            val group = awaitGroup(viewModel = viewModel, group = TaskGroup.MOVED)
                .groups.single { it.group == TaskGroup.MOVED }
            assertEquals(expected = "Rename all to match (1)", actual = group.bulkActions.single().label)
            group.bulkActions.single().onClick()
            app.idle()

            viewModel.state.first { state -> state.groups.none { it.group == TaskGroup.MOVED } }
            assertEquals(expected = "alpha", actual = app.read(file = app.sync.resolve("a.pdf")))
            assertFalse(actual = Files.exists(app.sync.resolve("older/renamed.pdf")))
        }
    }

    @Test
    fun `a copy under another name goes to the trash after confirming, the document stays`() {
        UiTest.run { app ->
            adoptOne(app = app)
            Files.copy(app.primary.resolve("a.pdf"), app.primary.resolve("a copy.pdf"))
            val viewModel = DashboardViewModel(services = app.services, session = app.session, platform = app.platform)
            app.session.scan()
            app.idle()

            val row = rowOf(state = awaitGroup(viewModel = viewModel, group = TaskGroup.DUPLICATES))
            row.buttons.first().onClick()
            assertTrue(actual = Files.exists(app.primary.resolve("a copy.pdf")))
            viewModel.dialogs.confirm()
            app.idle()

            viewModel.state.first { state -> state.groups.none { it.group == TaskGroup.DUPLICATES } }
            assertFalse(actual = Files.exists(app.primary.resolve("a copy.pdf")))
            assertEquals(expected = "alpha", actual = app.read(file = app.primary.resolve("a.pdf")))
        }
    }

    @Test
    fun `remove all duplicates keeps one document and trashes the other copy in every folder`() {
        UiTest.run { app ->
            app.write(file = app.primary.resolve("a.pdf"), content = "alpha")
            app.write(file = app.primary.resolve("b.pdf"), content = "alpha")
            app.configure()
            val viewModel = DashboardViewModel(services = app.services, session = app.session, platform = app.platform)
            awaitGroup(viewModel = viewModel, group = TaskGroup.NEW_FILES)
            val changes = assertNotNull(actual = app.session.state.value.lastScan).changes
            changes.filterIsInstance<DetectedChange.NewFile>().forEach {
                app.services.documents.adoptNewFile(change = it)
            }
            app.session.scan()
            app.idle()

            // Waits for the dashboard of the new scan, in which the adopted files are no longer new.
            val group = viewModel.state.first { state ->
                state.groups.any { it.group == TaskGroup.DUPLICATES && it.rows.size == 1 }
            }.groups.single { it.group == TaskGroup.DUPLICATES }
            val copies = group.rows.single().copies
            assertEquals(expected = setOf("a.pdf", "b.pdf"), actual = copies.map { it.path }.toSet())
            assertTrue(actual = copies.all { it.open.label == "Open" && it.keep.label == "Keep…" })
            group.bulkActions.single().onClick()
            viewModel.dialogs.confirm()
            app.idle()

            viewModel.state.first { it.documentCount == 1 }
            val left = listOf("a.pdf", "b.pdf").filter { Files.exists(app.primary.resolve(it)) }
            assertEquals(expected = 1, actual = left.size)
            val leftInSync = listOf("a.pdf", "b.pdf").filter { Files.exists(app.sync.resolve(it)) }
            assertEquals(expected = left, actual = leftInSync)
        }
    }

    private suspend fun adoptOne(app: TestApp) {
        app.write(file = app.primary.resolve("a.pdf"), content = "alpha")
        app.configure()
        val viewModel = DashboardViewModel(services = app.services, session = app.session, platform = app.platform)
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
