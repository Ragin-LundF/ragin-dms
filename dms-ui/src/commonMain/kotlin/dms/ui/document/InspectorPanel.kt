package dms.ui.document

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import dms.application.port.state.model.ActivityEntryModel
import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.document.details.NotesAndLinksSections
import dms.ui.document.model.InspectorTab
import dms.ui.document.suggestion.SuggestionCard
import dms.ui.document.suggestion.SuggestionValues
import dms.ui.document.tab.DetailsTab
import dms.ui.document.tab.HistoryTab
import dms.ui.document.tab.InspectorTabs
import dms.ui.library.model.CategoryEntryModel
import dms.ui.pdf.PdfSection
import dms.ui.theme.Spacing

/**
 * Everything about one document: a preview, the actions, a suggestion of the AI that waits for review,
 * and the tabs Details, Notes & links, Pages and History with their editors.
 */
@Composable
fun InspectorPanel(
    document: DocumentModel,
    categories: List<CategoryEntryModel>,
    allTags: List<TagNameModel>,
    folderId: LibraryFolderIdModel?,
    busyReason: String?,
    aiBusyReason: String?,
    viewModel: InspectorViewModel,
    onClose: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val catalog by viewModel.catalog.collectAsState()
    val feed by viewModel.feed.state.collectAsState()
    val selected by viewModel.tab.collectAsState()
    val tab = viewModel.shownTab(selected = selected, document = document)
    Surface(modifier = modifier, color = MaterialTheme.colorScheme.surface) {
        Column(
            modifier = Modifier.verticalScroll(state = rememberScrollState()).padding(all = Spacing.large),
            verticalArrangement = Arrangement.spacedBy(space = Spacing.medium),
        ) {
            InspectorHeader(
                document = document,
                folderId = folderId,
                thumbnails = viewModel.thumbnails,
                onClose = onClose,
            )
            DocumentActions(
                document = document,
                busyReason = busyReason,
                aiBusyReason = aiBusyReason,
                viewModel = viewModel,
            )
            feed.suggestions[document.id]?.let { suggestion ->
                catalog?.let { shown ->
                    SuggestionCard(
                        values = SuggestionValues.of(suggestion = suggestion, catalog = shown),
                        busyReason = busyReason,
                        onReview = { viewModel.reviewSuggestion(suggestion = suggestion) },
                        onDismiss = { viewModel.dismissSuggestion(document = document) },
                    )
                }
            }
            InspectorTabs(
                selected = tab,
                disabled = viewModel.disabledTabs(document = document),
                onSelect = viewModel::selectTab,
            )
            catalog?.let { shown ->
                TabContent(
                    tab = tab,
                    document = document,
                    catalog = shown,
                    categories = categories,
                    allTags = allTags,
                    busyReason = busyReason,
                    activity = feed.activity,
                    viewModel = viewModel,
                )
            }
        }
    }
}

@Composable
private fun TabContent(
    tab: InspectorTab,
    document: DocumentModel,
    catalog: CatalogModel,
    categories: List<CategoryEntryModel>,
    allTags: List<TagNameModel>,
    busyReason: String?,
    activity: List<ActivityEntryModel>,
    viewModel: InspectorViewModel,
) {
    when (tab) {
        InspectorTab.DETAILS -> DetailsTab(
            document = document,
            catalog = catalog,
            categories = categories,
            allTags = allTags,
            busyReason = busyReason,
            viewModel = viewModel,
        )

        InspectorTab.NOTES_AND_LINKS -> NotesAndLinksSections(
            document = document,
            catalog = catalog,
            busyReason = busyReason,
            controller = viewModel.details,
            onOpenDocument = viewModel::open,
        )

        InspectorTab.PAGES -> PdfSection(
            document = document,
            busyReason = busyReason,
            pages = viewModel.pages,
            actions = viewModel.pdf,
        )

        InspectorTab.HISTORY -> HistoryTab(entries = viewModel.historyOf(document = document, activity = activity))
    }
}
