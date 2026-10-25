package dms.ui.document.quicklook

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.hasSetTextAction
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performKeyInput
import androidx.compose.ui.test.performTextReplacement
import androidx.compose.ui.test.pressKey
import androidx.compose.ui.test.v2.runComposeUiTest
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.document.model.QuickFact
import dms.ui.document.model.QuickFactsModel
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

@OptIn(ExperimentalTestApi::class)
class QuickFactListTest {

    private val facts = QuickFactsModel(
        category = "Home",
        tags = listOf(TagNameModel.of(raw = "invoice")),
        correspondent = "Sample Energy",
        documentDate = "2025-10-02",
        amount = null,
        due = null,
    )

    @Test
    fun `a click on a value edits it in place, and Enter saves what was typed`() {
        val saved = mutableListOf<Pair<QuickFact, String>>()
        runComposeUiTest {
            setContent { FactsUnderTest(onSave = { fact, typed -> saved += fact to typed }) }

            onNodeWithText(text = "Sample Energy").performClick()
            onNode(matcher = hasSetTextAction()).performTextReplacement(text = "Sample Office")
            onNode(matcher = hasSetTextAction()).performKeyInput { pressKey(key = Key.Enter) }
        }
        assertEquals(expected = listOf(QuickFact.CORRESPONDENT to "Sample Office"), actual = saved)
    }

    @Test
    fun `Esc and an unchanged value end the edit without saving`() {
        val saved = mutableListOf<Pair<QuickFact, String>>()
        var editing: QuickFact? = QuickFact.DATE
        runComposeUiTest {
            setContent {
                FactsUnderTest(onSave = { fact, typed -> saved += fact to typed }, onEditing = { editing = it })
            }

            onNodeWithText(text = "2025-10-02").performClick()
            onNode(matcher = hasSetTextAction()).performKeyInput { pressKey(key = Key.Enter) }
            waitForIdle()
            val afterUnchanged = editing
            onNodeWithText(text = "2025-10-02").performClick()
            onNode(matcher = hasSetTextAction()).performTextReplacement(text = "2025-10-03")
            onNode(matcher = hasSetTextAction()).performKeyInput { pressKey(key = Key.Escape) }
            waitForIdle()

            assertNull(actual = afterUnchanged)
            onNodeWithText(text = "2025-10-02").assertExists()
        }
        assertNull(actual = editing)
        assertEquals(expected = emptyList(), actual = saved)
    }

    @Composable
    private fun FactsUnderTest(onSave: (QuickFact, String) -> Unit, onEditing: (QuickFact?) -> Unit = {}) {
        var editing by remember { mutableStateOf<QuickFact?>(value = null) }
        val change = { fact: QuickFact? ->
            editing = fact
            onEditing(fact)
        }
        QuickFactList(
            facts = facts,
            categories = emptyList(),
            allTags = emptyList(),
            editing = editing,
            busyReason = null,
            onEdit = { change(it) },
            onSave = onSave,
            onCancel = { change(null) },
            onPickCategory = {},
            onRemoveTag = {},
        )
    }
}
