package dms.ui.dashboard.section

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Modifier
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.dashboard.DashboardViewModel
import dms.ui.dashboard.attention.AttentionHandler
import dms.ui.dashboard.attention.AttentionRows
import dms.ui.dashboard.header.DashboardHeader
import dms.ui.dashboard.model.DashboardUiStateModel
import dms.ui.dashboard.stats.StatColumns
import dms.ui.dashboard.stats.model.StatLink
import dms.ui.dashboard.widgets.AiBand
import dms.ui.dashboard.widgets.AiBandContent
import dms.ui.dashboard.widgets.StatRow
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.YearFilterModel
import dms.ui.theme.Spacing

/**
 * The dashboard's sections one below the other: the header with the main actions, the headline
 * figures, what the AI can do, what needs the user beside the recent documents, quick access, the
 * charts and the state of the folders. Clicks open the library at what was clicked.
 */
@Composable
fun DashboardOverview(
    state: DashboardUiStateModel,
    busyReason: String?,
    viewModel: DashboardViewModel,
    onOpen: (LibrarySelection, String) -> Unit,
    onOpenYear: (YearFilterModel) -> Unit,
    onOpenDocument: (DocumentIdModel) -> Unit,
    onShowAllTasks: () -> Unit,
    onOpenInbox: () -> Unit,
) {
    val everything = LibrarySelection.Folder(path = RelativePathModel.ROOT)
    val scans by viewModel.organize.scans.collectAsState()
    val attentionHandler = rememberAttentionHandler(
        viewModel = viewModel,
        onOpenInbox = onOpenInbox,
        onOpenDocument = onOpenDocument,
    )
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(state = rememberScrollState())
            .padding(all = Spacing.screen),
        verticalArrangement = Arrangement.spacedBy(space = Spacing.large),
    ) {
        DashboardHeader(state = state)
        StatRow(columns = StatColumns.of(state = state)) { link ->
            when (link) {
                StatLink.UNCATEGORIZED -> onOpen(LibrarySelection.Uncategorized, "")
                StatLink.INBOX -> onOpenInbox()
            }
        }
        AiBandContent.of(state = state)?.let { band ->
            AiBand(band = band, onClassify = viewModel.aiBatches::classifyAll, onTag = viewModel.aiBatches::tagAll)
        }
        FocusSection(
            state = state,
            attention = AttentionRows.of(state = state, scans = scans, handler = attentionHandler),
            busyReason = busyReason,
            thumbnails = viewModel.thumbnails,
            onShowAllTasks = onShowAllTasks,
            onOpenDocument = onOpenDocument,
            onOpenRecent = { onOpen(LibrarySelection.Recent, "") },
        )
        QuickAccessSection(
            stats = state.stats,
            onOpenCategory = { id -> onOpen(LibrarySelection.Category(categoryId = CategoryIdModel(value = id)), "") },
            onOpenUncategorized = { onOpen(LibrarySelection.Uncategorized, "") },
            onOpenTag = { tag -> onOpen(LibrarySelection.Tag(tag = TagNameModel.of(raw = tag)), "") },
            onOpenYear = { id -> YearFilterModel.parse(text = id)?.let(onOpenYear) },
        )
        InsightsSection(stats = state.stats, onSearch = { query -> onOpen(everything, query) })
        StatusSection(state = state)
    }
}

/** The answers of "Needs your attention"; stays the same object and always calls the latest callbacks. */
@Composable
private fun rememberAttentionHandler(
    viewModel: DashboardViewModel,
    onOpenInbox: () -> Unit,
    onOpenDocument: (DocumentIdModel) -> Unit,
): AttentionHandler {
    val latestOpenInbox by rememberUpdatedState(newValue = onOpenInbox)
    val latestOpenDocument by rememberUpdatedState(newValue = onOpenDocument)
    return remember(viewModel) {
        object : AttentionHandler {
            override fun openInbox() {
                latestOpenInbox()
            }

            override fun analyzeInbox() {
                viewModel.analyzeInbox()
            }

            override fun openDocument(documentId: DocumentIdModel) {
                latestOpenDocument(documentId)
            }

            override fun findScans() {
                viewModel.organize.findScans()
            }

            override fun makeScansSearchable() {
                viewModel.organize.makeScansSearchable()
            }

            override fun exportCalendar() {
                viewModel.organize.exportCalendar()
            }
        }
    }
}
