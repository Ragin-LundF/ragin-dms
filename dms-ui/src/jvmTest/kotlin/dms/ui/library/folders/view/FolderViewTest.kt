package dms.ui.library.folders.view

import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.SemanticsNodeInteraction
import androidx.compose.ui.test.assertIsFocused
import androidx.compose.ui.test.assertIsNotFocused
import androidx.compose.ui.test.hasSetTextAction
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.v2.runComposeUiTest
import dms.domain.document.model.ContentHashModel
import dms.domain.document.model.DocumentIdModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.RelativePathModel
import dms.ui.components.dialog.DialogController
import dms.ui.library.drag.DocumentDrag
import dms.ui.library.drag.model.DragItem
import dms.ui.library.drag.model.DropTarget
import dms.ui.library.drag.model.FolderDropPlace
import dms.ui.library.folders.FolderBrowser
import dms.ui.library.state.DiskFolders
import dms.ui.support.TestApp
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import java.time.Instant
import kotlin.test.Test
import kotlin.test.assertEquals

@OptIn(ExperimentalTestApi::class)
class FolderViewTest {

    private val document = DocumentModel(
        id = DocumentIdModel(value = "power"),
        path = RelativePathModel.of(raw = "Home/Energy/power.txt"),
        contentHash = ContentHashModel(value = "0".repeat(n = 64)),
        size = 1,
        addedAt = Instant.parse("2025-01-01T10:00:00Z"),
    )

    @Test
    fun `a document dropped on a breadcrumb segment goes to that folder`() {
        val scope = CoroutineScope(context = SupervisorJob() + Dispatchers.Unconfined)
        val app = TestApp(scope = scope)
        val drops = mutableListOf<Pair<DragItem, DropTarget>>()
        val drag = DocumentDrag(onDrop = { item, target -> drops += item to target })
        val browser = FolderBrowser(
            services = app.services,
            session = app.session,
            dialogs = DialogController(),
            disk = DiskFolders(services = app.services, session = app.session, scope = scope),
            scope = scope,
        )
        browser.open(path = RelativePathModel.of(raw = "Home/Energy"))
        // Hidden, so "Library" names only the first segment and not the tree's top row too.
        browser.treeLayout.toggleFolded()
        try {
            runComposeUiTest {
                setContent {
                    FolderView(
                        browser = browser,
                        drag = drag,
                        platform = app.platform,
                        onAddFiles = {},
                        onDropFiles = { _, _ -> },
                        searchFocusRequests = 0,
                        documents = {},
                    )
                }

                dropOn(drag = drag, node = onNodeWithText(text = "Home"))
                dropOn(drag = drag, node = onNodeWithText(text = "Library"))
            }
        } finally {
            scope.cancel()
            app.close()
        }

        val item = DragItem.DraggedDocument(document = document)
        val home = RelativePathModel.of(raw = "Home")
        assertEquals(
            expected = listOf<Pair<DragItem, DropTarget>>(
                item to DropTarget.Folder(path = home, place = FolderDropPlace.BREADCRUMB),
                item to DropTarget.Folder(path = RelativePathModel.ROOT, place = FolderDropPlace.BREADCRUMB),
            ),
            actual = drops,
        )
    }

    @Test
    fun `a search request focuses the field of the open folder and typing searches in it`() {
        val scope = CoroutineScope(context = SupervisorJob() + Dispatchers.Unconfined)
        val app = TestApp(scope = scope)
        val browser = FolderBrowser(
            services = app.services,
            session = app.session,
            dialogs = DialogController(),
            disk = DiskFolders(services = app.services, session = app.session, scope = scope),
            scope = scope,
        )
        browser.open(path = RelativePathModel.of(raw = "Home"))
        val requests = mutableIntStateOf(value = 0)
        try {
            runComposeUiTest {
                setContent {
                    FolderView(
                        browser = browser,
                        drag = DocumentDrag(onDrop = { _, _ -> }),
                        platform = app.platform,
                        onAddFiles = {},
                        onDropFiles = { _, _ -> },
                        searchFocusRequests = requests.intValue,
                        documents = {},
                    )
                }
                onNode(matcher = hasSetTextAction()).assertIsNotFocused()

                requests.intValue = 1
                waitForIdle()
                onNode(matcher = hasSetTextAction()).assertIsFocused().performTextInput(text = "power")
            }
            assertEquals(expected = "power", actual = browser.state.value.query)
        } finally {
            scope.cancel()
            app.close()
        }
    }

    private fun dropOn(drag: DocumentDrag, node: SemanticsNodeInteraction) {
        val bounds = node.fetchSemanticsNode().boundsInRoot
        drag.start(item = DragItem.DraggedDocument(document = document), at = bounds.center)
        drag.move(by = Offset.Zero)
        drag.finish()
    }
}
