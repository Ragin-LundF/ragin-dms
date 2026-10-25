package dms.ui.components.layout

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.doubleClick
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performMouseInput
import androidx.compose.ui.test.v2.runComposeUiTest
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

@OptIn(ExperimentalTestApi::class)
class ResizeHandleTest {

    @Test
    fun `a drag reports its steps and its end once, and a double-click asks for a reset`() {
        val steps = mutableListOf<Dp>()
        var ends = 0
        var resets = 0
        runComposeUiTest {
            setContent {
                Row(modifier = Modifier.height(height = 200.dp)) {
                    ResizeHandle(
                        onResize = { steps += it },
                        modifier = Modifier.testTag(tag = "handle"),
                        onResizeEnd = { ends++ },
                        onDoubleClick = { resets++ },
                    )
                }
            }
            val handle = onNodeWithTag(testTag = "handle")

            handle.performMouseInput {
                press()
                moveBy(delta = Offset(x = 30f, y = 0f))
                moveBy(delta = Offset(x = 30f, y = 0f))
                release()
            }
            handle.performMouseInput { doubleClick() }
        }
        assertTrue(actual = steps.sumOf { it.value.toDouble() } > 0.0)
        assertEquals(expected = 1, actual = ends)
        assertEquals(expected = 1, actual = resets)
    }
}
