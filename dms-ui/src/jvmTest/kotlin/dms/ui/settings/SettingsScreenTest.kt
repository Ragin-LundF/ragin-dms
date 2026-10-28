package dms.ui.settings

import androidx.compose.ui.input.key.Key
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.ComposeUiTest
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.SemanticsNodeInteraction
import androidx.compose.ui.test.assertIsFocused
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.assertTextContains
import androidx.compose.ui.test.hasSetTextAction
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.isToggleable
import androidx.compose.ui.test.onFirst
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performKeyInput
import androidx.compose.ui.test.performTextReplacement
import androidx.compose.ui.test.pressKey
import androidx.compose.ui.test.v2.runComposeUiTest
import dms.ui.settings.model.SettingsSection
import dms.ui.support.TestApp
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.setMain
import kotlin.test.Test
import kotlin.test.assertEquals

@OptIn(ExperimentalTestApi::class)
class SettingsScreenTest {

    @Test
    fun `a configured library lists the six sections and shows only the library folders`() {
        settingsScreen(configured = true) {
            assertEquals(
                expected = listOf("Library folders", "Scanning", "Inbox", "Custom fields", "Matching rules", "AI"),
                actual = listedSections(),
            )
            section(label = "Library folders").assertIsSelected()
            onNodeWithText(text = FOLDERS_SUBTITLE).assertExists()
            onNodeWithText(text = SCANNING_SUBTITLE).assertDoesNotExist()
            onNodeWithText(text = AI_SWITCH).assertDoesNotExist()
        }
    }

    @Test
    fun `a click on a section shows only its card`() {
        settingsScreen(configured = true) {
            section(label = "AI").performClick()

            section(label = "AI").assertIsSelected()
            onNodeWithText(text = AI_SWITCH).assertExists()
            onNodeWithText(text = FOLDERS_SUBTITLE).assertDoesNotExist()
        }
    }

    @Test
    fun `tab reaches the sections and enter or space opens one`() {
        settingsScreen(configured = true) {
            pressKey(key = Key.Tab)
            section(label = "Library folders").assertIsFocused()
            pressKey(key = Key.Tab)
            section(label = "Scanning").assertIsFocused()
            pressKey(key = Key.Enter)

            section(label = "Scanning").assertIsSelected()
            onNodeWithText(text = SCANNING_SUBTITLE).assertExists()

            pressKey(key = Key.Tab)
            section(label = "Inbox").assertIsFocused()
            pressKey(key = Key.Spacebar)

            section(label = "Inbox").assertIsSelected()
            onNodeWithText(text = INBOX_SUBTITLE).assertExists()
        }
    }

    @Test
    fun `unsaved drafts survive switching to another section and back`() {
        settingsScreen(configured = true) {
            section(label = "Inbox").performClick()
            // The first switch is "Use an inbox folder"; the folder field is disabled while it is off.
            onAllNodes(matcher = isToggleable()).onFirst().performClick()
            textField(label = "Folder inside the library").performTextReplacement(text = "Unsorted mail")
            section(label = "AI").performClick()
            textField(label = "Server address").performTextReplacement(text = "http://ai.example:9")

            section(label = "Library folders").performClick()
            section(label = "Inbox").performClick()
            textField(label = "Folder inside the library").assertTextContains(value = "Unsorted mail")
            section(label = "AI").performClick()
            textField(label = "Server address").assertTextContains(value = "http://ai.example:9")
        }
    }

    @Test
    fun `an unconfigured library shows the welcome and AI cards without the section list`() {
        settingsScreen(configured = false) {
            onNodeWithText(text = "Welcome").assertExists()
            onNodeWithText(text = AI_SWITCH).assertExists()
            assertEquals(expected = emptyList(), actual = listedSections())
        }
    }

    @Test
    fun `sections that edit the library settings are listed only once they are loaded`() {
        assertEquals(
            expected = listOf(SettingsSection.FOLDERS, SettingsSection.SCANNING, SettingsSection.AI),
            actual = availableSections(settingsLoaded = false),
        )
        assertEquals(expected = SettingsSection.entries, actual = availableSections(settingsLoaded = true))
    }

    @OptIn(ExperimentalCoroutinesApi::class)
    private fun settingsScreen(configured: Boolean, block: ComposeUiTest.() -> Unit) {
        Dispatchers.setMain(dispatcher = UnconfinedTestDispatcher())
        val scope = CoroutineScope(context = SupervisorJob() + Dispatchers.Unconfined)
        val app = TestApp(scope = scope)
        try {
            val viewModel = SettingsViewModel(services = app.services, session = app.session, platform = app.platform)
            if (configured) {
                runBlocking {
                    app.configure()
                    viewModel.library.state.first { it.settings != null }
                }
            }
            runComposeUiTest {
                setContent { SettingsScreen(viewModel = viewModel) }
                block()
            }
        } finally {
            scope.cancel()
            app.close()
            Dispatchers.resetMain()
        }
    }

    private fun ComposeUiTest.listedSections(): List<String> {
        return onAllNodes(matcher = isSectionTab).fetchSemanticsNodes().map {
            it.config[SemanticsProperties.Text].joinToString()
        }
    }

    private fun ComposeUiTest.section(label: String): SemanticsNodeInteraction {
        return onNode(matcher = isSectionTab and hasText(text = label))
    }

    private fun ComposeUiTest.textField(label: String): SemanticsNodeInteraction {
        return onNode(matcher = hasSetTextAction() and hasText(text = label))
    }

    private fun ComposeUiTest.pressKey(key: Key) {
        onRoot().performKeyInput { pressKey(key = key) }
    }

    private companion object {
        val isSectionTab = SemanticsMatcher.expectValue(key = SemanticsProperties.Role, expectedValue = Role.Tab)
        const val FOLDERS_SUBTITLE = "Every change is written to the primary first, then to each sync folder"
        const val SCANNING_SUBTITLE = "What happens with files the library does not know yet"
        const val INBOX_SUBTITLE = "A folder in your library where new documents arrive, on every device"
        const val AI_SWITCH = "Use AI"
    }
}
