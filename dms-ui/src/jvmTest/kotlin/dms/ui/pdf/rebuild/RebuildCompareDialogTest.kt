package dms.ui.pdf.rebuild

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.v2.runComposeUiTest
import dms.application.pdf.rebuild.model.RebuildChoice
import dms.application.port.pdf.model.RebuiltFileModel
import dms.domain.document.model.ContentHashModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.RelativePathModel
import dms.domain.rebuild.KeepReason
import dms.domain.rebuild.model.RebuiltPageModel
import dms.ui.pdf.model.CompareView
import dms.ui.pdf.model.RebuildCompareStateModel
import kotlin.test.Test
import kotlin.test.assertEquals

@OptIn(ExperimentalTestApi::class)
class RebuildCompareDialogTest {

    private val state = RebuildCompareStateModel(
        documentId = DocumentIdModel(value = "doc-1"),
        path = RelativePathModel.of(raw = "scans/sample-scan.pdf"),
        rebuilt = RebuiltFileModel(
            id = "rebuild-1",
            pages = listOf(
                RebuiltPageModel(
                    linesAsText = 3,
                    linesTotal = 4,
                    angle = 0.0,
                    keptReasons = mapOf(KeepReason.NOT_SURE to 1),
                    regions = emptyList(),
                ),
                RebuiltPageModel(
                    linesAsText = 0,
                    linesTotal = 0,
                    angle = 0.0,
                    keptReasons = emptyMap(),
                    regions = emptyList(),
                ),
            ),
            size = 2_048,
            sourceHash = ContentHashModel(value = "0".repeat(n = 64)),
        ),
        originalSize = 4_096,
        original = ImageBitmap(width = 20, height = 30),
        rebuiltImage = ImageBitmap(width = 20, height = 30),
    )

    @Test
    fun `each button chooses its own way`() {
        val chosen = mutableListOf<RebuildChoice>()

        runComposeUiTest {
            setContent { Dialog(state = state, onChoose = { chosen += it }) }
            onNodeWithText(text = "Keep original").performClick()
            onNodeWithText(text = "Keep both").performClick()
            onNodeWithText(text = "Use rebuilt PDF").performClick()
        }

        assertEquals(
            expected = listOf(RebuildChoice.KEEP_ORIGINAL, RebuildChoice.KEEP_BOTH, RebuildChoice.REPLACE_WITH_REBUILD),
            actual = chosen,
        )
    }

    @Test
    fun `the summary names the lines, the sizes and why lines were kept, and next turns the page`() {
        val pages = mutableListOf<Int>()

        runComposeUiTest {
            setContent { Dialog(state = state, onPage = { pages += it }) }
            onNodeWithText(text = "3 of 4 lines on this page are real text", substring = true).assertExists()
            onNodeWithText(text = "1 × the AI was not sure of every character", substring = true).assertExists()
            onNodeWithContentDescription(label = "Next page").performClick()
        }

        assertEquals(expected = listOf(1), actual = pages)
    }

    @Test
    fun `flipped, only the rebuilt page shows`() {
        runComposeUiTest {
            setContent { Dialog(state = state.copy(view = CompareView.REBUILT)) }
            onNodeWithContentDescription(label = "Rebuilt page").assertExists()
            onNodeWithContentDescription(label = "Original page").assertDoesNotExist()
        }
    }

    @Composable
    private fun Dialog(
        state: RebuildCompareStateModel,
        onChoose: (RebuildChoice) -> Unit = {},
        onPage: (Int) -> Unit = {},
    ) {
        RebuildCompare(
            state = state,
            onChoose = onChoose,
            onPage = onPage,
            onFlip = {},
            onSideBySide = {},
            onOverlay = {},
        )
    }
}
