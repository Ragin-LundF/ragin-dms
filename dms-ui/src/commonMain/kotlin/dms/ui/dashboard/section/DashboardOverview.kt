package dms.ui.dashboard.section

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.domain.document.DocumentId
import dms.domain.library.RelativePath
import dms.domain.taxonomy.CategoryId
import dms.domain.taxonomy.TagName
import dms.ui.dashboard.DashboardViewModel
import dms.ui.dashboard.hero.DashboardHero
import dms.ui.dashboard.model.DashboardUiState
import dms.ui.dashboard.widgets.StatRow
import dms.ui.library.model.LibrarySelection
import dms.ui.library.model.YearFilter
import dms.ui.theme.Spacing

/**
 * The dashboard's sections one below the other: the main actions, the headline figures, quick
 * access, the work that waits, and the charts. Clicks open the library at what was clicked.
 */
@Composable
fun DashboardOverview(
    state: DashboardUiState,
    busyReason: String?,
    viewModel: DashboardViewModel,
    onChooseFiles: () -> Unit,
    onOpen: (LibrarySelection, String) -> Unit,
    onOpenYear: (YearFilter) -> Unit,
    onOpenDocument: (DocumentId) -> Unit,
    onShowAllTasks: () -> Unit,
) {
    val everything = LibrarySelection.Folder(path = RelativePath.ROOT)
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
        QuickAccessSection(
            stats = state.stats,
            onOpenDocument = onOpenDocument,
            onOpenRecent = { onOpen(LibrarySelection.Recent, "") },
            onOpenCategory = { id -> onOpen(LibrarySelection.Category(categoryId = CategoryId(value = id)), "") },
            onOpenUncategorized = { onOpen(LibrarySelection.Uncategorized, "") },
            onOpenTag = { tag -> onOpen(LibrarySelection.Tag(tag = TagName.of(raw = tag)), "") },
        )
        WorkSection(
            state = state,
            busyReason = busyReason,
            onFilter = viewModel::setFilter,
            onFullView = onShowAllTasks,
        )
        InsightsSection(
            stats = state.stats,
            onSearch = { query -> onOpen(everything, query) },
            onOpenYear = { id -> YearFilter.parse(text = id)?.let(onOpenYear) },
        )
    }
}
