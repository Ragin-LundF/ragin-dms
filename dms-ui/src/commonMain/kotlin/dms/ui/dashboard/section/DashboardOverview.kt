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
import androidx.compose.ui.Modifier
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.RelativePathModel
import dms.domain.taxonomy.model.CategoryIdModel
import dms.domain.taxonomy.model.TagNameModel
import dms.ui.dashboard.DashboardViewModel
import dms.ui.dashboard.hero.DashboardHero
import dms.ui.dashboard.model.DashboardUiStateModel
import dms.ui.dashboard.widgets.StatRow
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.YearFilterModel
import dms.ui.theme.Spacing

/**
 * The dashboard's sections one below the other: the main actions, the headline figures, the inbox,
 * quick access, the work that waits, and the charts. Clicks open the library at what was clicked.
 */
@Composable
fun DashboardOverview(
    state: DashboardUiStateModel,
    busyReason: String?,
    viewModel: DashboardViewModel,
    onChooseFiles: () -> Unit,
    onOpen: (LibrarySelection, String) -> Unit,
    onOpenYear: (YearFilterModel) -> Unit,
    onOpenDocument: (DocumentIdModel) -> Unit,
    onShowAllTasks: () -> Unit,
    onOpenInbox: () -> Unit,
) {
    val everything = LibrarySelection.Folder(path = RelativePathModel.ROOT)
    val scans by viewModel.organize.scans.collectAsState()
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(state = rememberScrollState())
            .padding(all = Spacing.xxlarge),
        verticalArrangement = Arrangement.spacedBy(space = Spacing.xlarge),
    ) {
        DashboardHero(
            state = state,
            busyReason = busyReason,
            onAddFiles = onChooseFiles,
            onScan = viewModel::scan,
            onSync = viewModel::syncToAllDestinations,
            onMerge = viewModel::mergeAllFolders,
            onClassifyAll = viewModel.aiBatches::classifyAll,
            onTagAll = viewModel.aiBatches::tagAll,
            onSearch = { query -> onOpen(everything, query) },
        )
        StatRow(state = state, onOpenUncategorized = { onOpen(LibrarySelection.Uncategorized, "") })
        OrganizeSection(
            state = state,
            onOpenInbox = onOpenInbox,
            onAnalyzeInbox = viewModel::analyzeInbox,
            onOpenDocument = onOpenDocument,
            onExportCalendar = viewModel.organize::exportCalendar,
            scans = scans,
            onFindScans = viewModel.organize::findScans,
            onMakeSearchable = viewModel.organize::makeScansSearchable,
        )
        QuickAccessSection(
            stats = state.stats,
            onOpenDocument = onOpenDocument,
            onOpenRecent = { onOpen(LibrarySelection.Recent, "") },
            onOpenCategory = { id -> onOpen(LibrarySelection.Category(categoryId = CategoryIdModel(value = id)), "") },
            onOpenUncategorized = { onOpen(LibrarySelection.Uncategorized, "") },
            onOpenTag = { tag -> onOpen(LibrarySelection.Tag(tag = TagNameModel.of(raw = tag)), "") },
        )
        WorkSection(
            state = state,
            busyReason = busyReason,
            thumbnails = viewModel.thumbnails,
            onFilter = viewModel::setFilter,
            onFullView = onShowAllTasks,
        )
        InsightsSection(
            stats = state.stats,
            onSearch = { query -> onOpen(everything, query) },
            onOpenYear = { id -> YearFilterModel.parse(text = id)?.let(onOpenYear) },
        )
    }
}
