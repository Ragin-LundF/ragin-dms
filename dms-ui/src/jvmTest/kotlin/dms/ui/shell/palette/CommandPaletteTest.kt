package dms.ui.shell.palette

import dms.ui.shell.palette.model.PaletteAction
import dms.ui.shell.palette.model.PaletteItemModel
import dms.ui.support.UiTest
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.runCurrent
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class CommandPaletteTest {

    @Test
    fun `opening shows every action, typing narrows them and starts at the first line`() {
        UiTest.run { app ->
            val palette = CommandPalette(services = app.services, scope = backgroundScope)

            palette.open()
            runCurrent()
            assertTrue(actual = palette.state.value.open)
            assertEquals(expected = PaletteAction.entries.size, actual = palette.state.value.items.size)

            palette.move(steps = 2)
            palette.type(text = "sync")
            runCurrent()

            assertEquals(expected = 0, actual = palette.state.value.selected)
            assertEquals(
                expected = listOf(
                    PaletteItemModel.Run(action = PaletteAction.SYNC_ALL),
                    PaletteItemModel.AskLibrary(question = "sync"),
                ),
                actual = palette.state.value.items,
            )
        }
    }

    @Test
    fun `the arrow keys wrap around, and choosing returns the line and closes`() {
        UiTest.run { app ->
            val palette = CommandPalette(services = app.services, scope = backgroundScope)
            palette.open(initial = "sync")

            palette.move(steps = -1)
            val chosen = palette.choose()
            runCurrent()

            assertEquals(expected = PaletteItemModel.AskLibrary(question = "sync"), actual = chosen)
            assertFalse(actual = palette.state.value.open)
        }
    }

    @Test
    fun `asking takes what is typed, and an empty query asks nothing`() {
        UiTest.run { app ->
            val palette = CommandPalette(services = app.services, scope = backgroundScope)
            palette.open()

            assertNull(actual = palette.ask())

            palette.type(text = " power bill ")
            assertEquals(expected = PaletteItemModel.AskLibrary(question = "power bill"), actual = palette.ask())
            runCurrent()
            assertFalse(actual = palette.state.value.open)
        }
    }
}
