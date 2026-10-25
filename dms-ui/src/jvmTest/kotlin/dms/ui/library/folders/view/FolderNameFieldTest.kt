package dms.ui.library.folders.view

import androidx.compose.ui.input.key.Key
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.hasSetTextAction
import androidx.compose.ui.test.performKeyInput
import androidx.compose.ui.test.performTextReplacement
import androidx.compose.ui.test.pressKey
import androidx.compose.ui.test.v2.runComposeUiTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

@OptIn(ExperimentalTestApi::class)
class FolderNameFieldTest {

    @Test
    fun `Enter saves the typed name once, also when the field loses the focus afterwards`() {
        val typed = mutableListOf<String>()
        val saved = mutableListOf<String>()
        var cancelled = false
        runComposeUiTest {
            setContent {
                FolderNameField(
                    initial = "New folder",
                    onType = { typed += it },
                    onSave = { saved += it },
                    onCancel = { cancelled = true },
                )
            }
            val field = onNode(matcher = hasSetTextAction())

            field.performTextReplacement(text = "Energy")
            field.performKeyInput { pressKey(key = Key.Enter) }
            field.performKeyInput { pressKey(key = Key.Escape) }
        }

        assertEquals(expected = "Energy", actual = typed.lastOrNull())
        assertEquals(expected = listOf("Energy"), actual = saved)
        assertFalse(actual = cancelled)
    }

    @Test
    fun `Esc cancels without saving`() {
        val saved = mutableListOf<String>()
        var cancelled = false
        runComposeUiTest {
            setContent {
                FolderNameField(
                    initial = "New folder",
                    onType = {},
                    onSave = { saved += it },
                    onCancel = { cancelled = true },
                )
            }

            onNode(matcher = hasSetTextAction()).performKeyInput { pressKey(key = Key.Escape) }
        }

        assertTrue(actual = saved.isEmpty())
        assertTrue(actual = cancelled)
    }
}
