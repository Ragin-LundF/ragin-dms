package dms.ui.dashboard.widgets

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.v2.runComposeUiTest
import dms.ui.dashboard.widgets.model.AiBandModel
import kotlin.test.Test
import kotlin.test.assertEquals

@OptIn(ExperimentalTestApi::class)
class AiBandTest {

    @Test
    fun `a button whose count is 0 is not shown, the other one runs its batch`() {
        var tagged = 0
        runComposeUiTest {
            setContent {
                AiBand(
                    band = AiBandModel(
                        headline = "3 documents have no tags yet",
                        classifyLabel = null,
                        tagLabel = "Tag 3 with AI",
                        busyReason = null,
                    ),
                    onClassify = {},
                    onTag = { tagged++ },
                )
            }

            onNodeWithText(text = "3 documents have no tags yet").assertExists()
            onNodeWithText(text = "Classify", substring = true).assertDoesNotExist()
            onNodeWithText(text = "Tag 3 with AI").performClick()
        }
        assertEquals(expected = 1, actual = tagged)
    }
}
