package dms.ui.library.documents.grid

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.v2.runComposeUiTest
import dms.ui.library.documents.model.DocumentClick
import dms.ui.library.model.DocumentRowModel
import dms.ui.preview.ThumbnailLoader
import dms.ui.support.SyntheticCatalog
import dms.ui.support.TestApp
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancel
import kotlin.test.Test
import kotlin.test.assertEquals

@OptIn(ExperimentalTestApi::class)
class DocumentCardTest {

    private val row = DocumentRowModel(
        document = SyntheticCatalog.document(
            id = "1",
            path = "a.pdf",
            title = "Water bill",
            correspondent = "Sample Utility",
        ),
        categoryName = "Energy",
    )

    @Test
    fun `the circle ticks the document without opening it, a click on the card opens it`() {
        val scope = CoroutineScope(context = Job())
        val app = TestApp(scope = scope)
        val clicks = mutableListOf<DocumentClick>()
        var toggles = 0
        try {
            runComposeUiTest {
                setContent {
                    DocumentCard(
                        row = row,
                        selected = false,
                        inspected = false,
                        selecting = true,
                        folderId = null,
                        thumbnails = ThumbnailLoader(services = app.services, platform = app.platform),
                        onClick = { clicks += it },
                        onToggle = { toggles++ },
                    )
                }
                onNodeWithText(text = "Sample Utility · Energy").assertExists()

                onNodeWithContentDescription(label = "Select Water bill").performClick()
                onNodeWithText(text = "Water bill").performClick()
            }
        } finally {
            scope.cancel()
            app.close()
        }
        assertEquals(expected = 1, actual = toggles)
        assertEquals(expected = listOf(DocumentClick.PLAIN), actual = clicks)
    }
}
