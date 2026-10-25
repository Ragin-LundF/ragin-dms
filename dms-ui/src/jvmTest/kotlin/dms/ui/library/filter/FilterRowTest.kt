package dms.ui.library.filter

import androidx.compose.runtime.Composable
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.v2.runComposeUiTest
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.library.filter.model.ActiveFilter
import dms.ui.library.model.LibraryUiStateModel
import dms.ui.library.model.YearFilterModel
import kotlin.test.Test
import kotlin.test.assertEquals

@OptIn(ExperimentalTestApi::class)
class FilterRowTest {

    @Test
    fun `active filters show as chips, and a click on one removes that filter`() {
        val tag = ActiveFilter.Tag(tag = TagNameModel.of(raw = "invoice"))
        val year = ActiveFilter.Year(year = YearFilterModel(year = 2025))
        val removed = mutableListOf<ActiveFilter>()
        runComposeUiTest {
            setContent {
                FilterRowUnderTest(
                    state = LibraryUiStateModel(filters = listOf(tag, year)),
                    onRemove = { removed += it },
                )
            }

            onNodeWithText(text = "tag: invoice").assertExists()
            onNodeWithContentDescription(label = "Remove year: 2025").performClick()
        }
        assertEquals(expected = listOf<ActiveFilter>(year), actual = removed)
    }

    @Test
    fun `save as smart folder is offered only while filters are active`() {
        runComposeUiTest {
            setContent { FilterRowUnderTest(state = LibraryUiStateModel()) }

            onNodeWithText(text = "Save as smart folder").assertDoesNotExist()
            onNodeWithText(text = "+ Filter").assertExists()
        }
        runComposeUiTest {
            val text = ActiveFilter.Text(query = "rechnung", inContent = true)
            setContent { FilterRowUnderTest(state = LibraryUiStateModel(filters = listOf(text))) }

            onNodeWithText(text = "\"rechnung\" in text").assertExists()
            onNodeWithText(text = "Save as smart folder").assertExists()
        }
    }

    @Composable
    private fun FilterRowUnderTest(state: LibraryUiStateModel, onRemove: (ActiveFilter) -> Unit = {}) {
        FilterRow(
            state = state,
            busyReason = null,
            onRemove = onRemove,
            onYear = {},
            onContentSearch = {},
            onLayout = {},
            onSaveSmartFolder = {},
            onNewFolder = {},
            onExport = {},
        )
    }
}
