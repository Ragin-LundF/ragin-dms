package dms.ui.document.quicklook

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import dms.domain.catalog.model.CatalogModel
import dms.domain.document.model.DocumentModel
import dms.domain.library.model.LibraryFolderIdModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.document.QuickLookViewModel
import dms.ui.document.model.QuickFact
import dms.ui.document.model.QuickLookStateModel
import dms.ui.document.suggestion.SuggestionCard
import dms.ui.library.model.CategoryEntryModel
import dms.ui.theme.LibrarySizes
import dms.ui.theme.Spacing

/**
 * The quick look of the document [QuickLookViewModel.quickLook] shows, as a sheet over the documents;
 * nothing while it is closed or the document left the catalog. [busyReason] and [aiBusyReason] say
 * why the library or the AI lane cannot take another run now.
 */
@Composable
fun QuickLookSheet(
    categories: List<CategoryEntryModel>,
    allTags: List<TagNameModel>,
    folderId: LibraryFolderIdModel?,
    busyReason: String?,
    aiBusyReason: String?,
    viewModel: QuickLookViewModel,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.quickLook.state.collectAsState()
    val catalog by viewModel.catalog.collectAsState()
    val document = state.shown?.let { catalog?.documents?.get(it) } ?: return
    QuickLookPanel(
        document = document,
        categories = categories,
        allTags = allTags,
        folderId = folderId,
        busyReason = busyReason,
        aiBusyReason = aiBusyReason,
        viewModel = viewModel,
        onClose = { viewModel.quickLook.close() },
        modifier = modifier,
    )
}

/** The sheet for [document]: the top bar, the preview and title, a pending suggestion, the key facts and more. */
@Composable
private fun QuickLookPanel(
    document: DocumentModel,
    categories: List<CategoryEntryModel>,
    allTags: List<TagNameModel>,
    folderId: LibraryFolderIdModel?,
    busyReason: String?,
    aiBusyReason: String?,
    viewModel: QuickLookViewModel,
    onClose: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val catalog by viewModel.catalog.collectAsState()
    val state by viewModel.quickLook.state.collectAsState()
    Surface(
        modifier = modifier.width(width = LibrarySizes.quickLookWidth).fillMaxHeight(),
        color = MaterialTheme.colorScheme.surface,
        shadowElevation = Spacing.small,
    ) {
        Row {
            VerticalDivider(color = MaterialTheme.colorScheme.outlineVariant)
            Column {
                QuickLookTopBar(
                    document = document,
                    busyReason = busyReason,
                    aiBusyReason = aiBusyReason,
                    viewModel = viewModel,
                    onClose = onClose,
                )
                HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
                Column(
                    modifier = Modifier
                        .verticalScroll(state = rememberScrollState())
                        .padding(horizontal = Spacing.large, vertical = Spacing.medium),
                    verticalArrangement = Arrangement.spacedBy(space = Spacing.medium),
                ) {
                    QuickLookHeader(
                        document = document,
                        folderId = folderId,
                        thumbnails = viewModel.thumbnails,
                        editingTitle = state.editing == QuickFact.TITLE,
                        busyReason = busyReason,
                        onEditTitle = { viewModel.quickLook.edit(fact = QuickFact.TITLE) },
                        onSaveTitle = {
                            viewModel.saveFact(document = document, fact = QuickFact.TITLE, typed = it)
                        },
                        onCancel = viewModel.quickLook::endEdit,
                    )
                    catalog?.let { shown ->
                        SheetBody(
                            document = document,
                            catalog = shown,
                            state = state,
                            categories = categories,
                            allTags = allTags,
                            busyReason = busyReason,
                            viewModel = viewModel,
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun SheetBody(
    document: DocumentModel,
    catalog: CatalogModel,
    state: QuickLookStateModel,
    categories: List<CategoryEntryModel>,
    allTags: List<TagNameModel>,
    busyReason: String?,
    viewModel: QuickLookViewModel,
) {
    val feed by viewModel.feed.state.collectAsState()
    feed.suggestions[document.id]?.let { suggestion ->
        SuggestionCard(
            values = viewModel.suggestedValues(suggestion = suggestion, catalog = catalog),
            busyReason = busyReason,
            onAccept = { viewModel.acceptSuggestion(suggestion = suggestion) },
            onEdit = { viewModel.reviewSuggestion(suggestion = suggestion) },
            onDismiss = { viewModel.dismissSuggestion(document = document) },
        )
    }
    QuickFactList(
        facts = viewModel.factsOf(document = document, catalog = catalog),
        categories = categories,
        allTags = allTags,
        editing = state.editing,
        busyReason = busyReason,
        onEdit = { viewModel.quickLook.edit(fact = it) },
        onSave = { fact, typed -> viewModel.saveFact(document = document, fact = fact, typed = typed) },
        onCancel = viewModel.quickLook::endEdit,
        onPickCategory = { viewModel.assignCategory(document = document, categoryId = it) },
        onRemoveTag = { viewModel.removeTag(document = document, tag = it) },
    )
    MoreSection(document = document, catalog = catalog, busyReason = busyReason, viewModel = viewModel)
}
