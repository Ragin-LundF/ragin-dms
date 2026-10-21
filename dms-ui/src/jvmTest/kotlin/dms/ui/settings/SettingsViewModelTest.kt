package dms.ui.settings

import dms.ui.session.model.LibraryStatus
import dms.ui.support.UiTest
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class SettingsViewModelTest {

    @Test
    fun `choosing the primary folder opens a new library there`() {
        UiTest.run { app ->
            val viewModel = SettingsViewModel(services = app.services, session = app.session, platform = app.platform)
            app.platform.nextFolder = app.primary.toString()

            viewModel.choosePrimaryFolder()
            app.idle()

            assertIs<LibraryStatus.Open>(value = app.session.state.value.status)
            assertEquals(expected = listOf(true), actual = viewModel.state.value.rows.map { it.isPrimary })
            assertTrue(actual = Files.exists(app.primary.resolve("dms-catalog.yaml")))
        }
    }

    @Test
    fun `an added sync folder receives the catalog`() {
        UiTest.run { app ->
            app.configure(withSync = false)
            val viewModel = SettingsViewModel(services = app.services, session = app.session, platform = app.platform)
            app.platform.nextFolder = app.sync.toString()

            viewModel.addSyncFolder()
            app.idle()

            assertEquals(
                expected = listOf("Primary", "sync"),
                actual = viewModel.state.value.rows.map { it.folder.displayName },
            )
            assertTrue(actual = Files.exists(app.sync.resolve("dms-catalog.yaml")))
        }
    }

    @Test
    fun `a folder cannot be added twice`() {
        UiTest.run { app ->
            app.configure()
            val viewModel = SettingsViewModel(services = app.services, session = app.session, platform = app.platform)
            app.platform.nextFolder = app.sync.toString()

            viewModel.addSyncFolder()
            app.idle()

            assertEquals(expected = 2, actual = viewModel.state.value.rows.size)
            assertEquals(expected = true, actual = app.session.state.value.message?.isError)
        }
    }

    @Test
    fun `removing a sync folder asks first and keeps its files`() {
        UiTest.run { app ->
            app.configure()
            val viewModel = SettingsViewModel(services = app.services, session = app.session, platform = app.platform)
            val syncRow = viewModel.state.value.rows.single { !it.isPrimary }

            viewModel.removeSyncFolder(folder = syncRow.folder)

            assertNotNull(actual = viewModel.dialogs.confirmation.value)
            assertEquals(expected = 2, actual = viewModel.state.value.rows.size)

            viewModel.dialogs.confirm()
            app.idle()

            assertNull(actual = viewModel.dialogs.confirmation.value)
            assertEquals(expected = 1, actual = viewModel.state.value.rows.size)
            assertTrue(actual = Files.exists(app.sync.resolve("dms-catalog.yaml")))
        }
    }
}
