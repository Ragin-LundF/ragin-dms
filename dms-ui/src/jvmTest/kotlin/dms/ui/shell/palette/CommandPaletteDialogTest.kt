package dms.ui.shell.palette

import androidx.compose.ui.input.key.Key
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.KeyInjectionScope
import androidx.compose.ui.test.hasSetTextAction
import androidx.compose.ui.test.performKeyInput
import androidx.compose.ui.test.pressKey
import androidx.compose.ui.test.v2.runComposeUiTest
import androidx.compose.ui.test.withKeyDown
import dms.ui.shell.palette.model.PaletteAction
import dms.ui.shell.palette.model.PaletteItemModel
import dms.ui.support.TestApp
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse

@OptIn(ExperimentalTestApi::class)
class CommandPaletteDialogTest {

    @Test
    fun `enter chooses the selected line and closes the palette`() {
        val chosen = run(typed = "go to library") { pressKey(key = Key.Enter) }

        assertEquals(
            expected = listOf<PaletteItemModel>(PaletteItemModel.Run(action = PaletteAction.OPEN_LIBRARY)),
            actual = chosen,
        )
    }

    @Test
    fun `command and enter asks the library what was typed`() {
        val chosen = run(typed = "power bill") { withKeyDown(key = Key.MetaLeft) { pressKey(key = Key.Enter) } }

        assertEquals(
            expected = listOf<PaletteItemModel>(PaletteItemModel.AskLibrary(question = "power bill")),
            actual = chosen,
        )
    }

    @Test
    fun `escape closes the palette without choosing`() {
        val chosen = run(typed = "scan") { pressKey(key = Key.Escape) }

        assertEquals(expected = emptyList(), actual = chosen)
    }

    /** Opens the palette with [typed], presses [keys] in its field and returns what was chosen. */
    private fun run(typed: String, keys: KeyInjectionScope.() -> Unit): List<PaletteItemModel> {
        val scope = CoroutineScope(context = SupervisorJob() + Dispatchers.Default)
        val app = TestApp(scope = scope)
        val chosen = mutableListOf<PaletteItemModel>()
        try {
            val palette = CommandPalette(services = app.services, session = app.session, scope = scope)
            palette.open(initial = typed)
            runComposeUiTest {
                setContent { CommandPaletteDialog(palette = palette, onChoose = { chosen += it }) }
                waitUntil { palette.state.value.open && palette.state.value.query == typed }

                onNode(matcher = hasSetTextAction()).performKeyInput(block = keys)
                waitUntil { !palette.state.value.open }
            }
            assertFalse(actual = palette.state.value.open)
        } finally {
            app.close()
            scope.cancel()
        }
        return chosen
    }
}
