package dms.ui.library.filter

import androidx.compose.foundation.layout.Column
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onLast
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performKeyInput
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.pressKey
import androidx.compose.ui.test.v2.runComposeUiTest
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.library.filter.model.Facet
import dms.ui.library.filter.model.FacetFilter
import dms.ui.library.model.LibraryViewStateModel
import dms.ui.library.search.model.ContentSearchStateModel
import dms.ui.library.state.LibraryStates
import dms.ui.session.model.SessionStateModel
import dms.ui.support.SyntheticCatalog
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneOffset
import kotlin.test.Test

/** The search field's tokens and the facet chips render the same filters, so they change together. */
@OptIn(ExperimentalTestApi::class)
class FacetChipTest {

    private val invoice = TagNameModel.of(raw = "invoice")
    private val catalog = SyntheticCatalog.catalog(
        SyntheticCatalog.document(
            id = "1",
            path = "a.pdf",
            tags = setOf(invoice),
            documentDate = LocalDate.parse("2025-03-01"),
        ),
    )

    @Test
    fun `removing a token clears its chip`() {
        runComposeUiTest {
            setContent { FiltersUnderTest(start = listOf(FacetFilter.Tag(tag = invoice))) }
            onNodeWithText(text = "Tag: invoice").assertExists()

            onNodeWithContentDescription(label = "Remove tag: invoice").performClick()

            onNodeWithText(text = "Tag: invoice").assertDoesNotExist()
            onNodeWithText(text = "Tag").assertExists()
        }
    }

    @Test
    fun `a value chosen in a chip's popover becomes a token, and choosing it again removes it`() {
        runComposeUiTest {
            setContent { FiltersUnderTest(start = emptyList()) }

            onNodeWithText(text = "Year").performClick()
            onNodeWithText(text = "2025").performClick()
            onNodeWithContentDescription(label = "Remove year: 2025").assertExists()
            onNodeWithText(text = "Year: 2025").performClick()
            onAllNodesWithText(text = "2025").onLast().performClick()

            onNodeWithContentDescription(label = "Remove year: 2025").assertDoesNotExist()
            onNodeWithText(text = "Year").assertExists()
        }
    }

    @Test
    fun `enter takes the suggested token and backspace in the empty field removes the last one`() {
        runComposeUiTest {
            setContent { FiltersUnderTest(start = emptyList()) }
            val field = onNodeWithContentDescription(label = "Search the documents")

            field.performTextInput(text = "2025")
            onNodeWithText(text = "+ year: 2025").assertExists()
            field.performKeyInput { pressKey(key = Key.Enter) }
            onNodeWithText(text = "Year: 2025").assertExists()
            field.performKeyInput { pressKey(key = Key.Backspace) }

            onNodeWithText(text = "Year: 2025").assertDoesNotExist()
            onNodeWithContentDescription(label = "Remove year: 2025").assertDoesNotExist()
        }
    }

    /** The field and the chips over one view state, derived as the view model derives it. */
    @Composable
    private fun FiltersUnderTest(start: List<FacetFilter>) {
        var view by remember { mutableStateOf(value = LibraryViewStateModel(filters = start)) }
        val state = LibraryStates.of(
            catalog = catalog,
            session = SessionStateModel(),
            view = view,
            search = ContentSearchStateModel(),
            smartMatches = null,
            selected = emptySet(),
            reviewIds = emptySet(),
            smartFolders = { emptyList() },
            now = Instant.parse("2025-06-30T12:00:00Z"),
            zone = ZoneOffset.UTC,
        )
        val add: (FacetFilter) -> Unit = { filter ->
            view = view.copy(filters = FilterTokens.with(filters = view.filters, filter = filter))
        }
        val remove: (FacetFilter) -> Unit = { view = view.copy(filters = view.filters - it) }
        Column {
            TokenSearchField(
                query = view.query,
                tokens = state.tokens,
                suggestions = state.suggestions,
                placeholder = "Search",
                focusRequests = 0,
                onQuery = { view = view.copy(query = it) },
                onRemove = remove,
                onTake = {
                    add(it)
                    view = view.copy(query = "")
                },
            )
            Facet.entries.forEach { facet -> FacetChip(facet = facet, state = state, onPick = add, onRemove = remove) }
        }
    }
}
