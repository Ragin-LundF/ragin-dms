package dms.ui.dashboard

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
import dms.domain.document.DocumentId
import dms.domain.library.RelativePath
import dms.domain.taxonomy.CategoryId
import dms.domain.taxonomy.TagName
import dms.ui.PlatformActions
import dms.ui.components.dialog.DialogHost
import dms.ui.components.layout.FileDropZone
import dms.ui.dashboard.hero.DashboardHero
import dms.ui.dashboard.section.InsightsSection
import dms.ui.dashboard.section.QuickAccessSection
import dms.ui.dashboard.section.WorkSection
import dms.ui.dashboard.widgets.StatRow
import dms.ui.library.model.LibrarySelection
import dms.ui.review.ReviewDialog
import dms.ui.theme.Spacing

/**
 * The home screen: the main actions, the headline figures, quick access by category, tag and
 * correspondent, the open tasks, and charts of the library. Files dropped on it go to the top level.
 */
@Composable
fun DashboardScreen(
    viewModel: DashboardViewModel,
    platform: PlatformActions,
    onAddFiles: (List<String>) -> Unit,
    onChooseFiles: () -> Unit,
    onOpen: (LibrarySelection, String) -> Unit,
    onOpenDocument: (DocumentId) -> Unit,
) {
    val state by viewModel.state.collectAsState()
    val busyReason = state.busy?.let { "Wait until \"$it\" finishes" }
    val everything = LibrarySelection.Folder(path = RelativePath.ROOT)
    FileDropZone(
        platform = platform,
        label = "The files go to the top level of every library folder",
        onDrop = onAddFiles,
        modifier = Modifier.fillMaxSize(),
    ) {
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
                onOpenCategory = { id -> onOpen(LibrarySelection.Category(categoryId = CategoryId(value = id)), "") },
                onOpenUncategorized = { onOpen(LibrarySelection.Uncategorized, "") },
                onOpenTag = { tag -> onOpen(LibrarySelection.Tag(tag = TagName.of(raw = tag)), "") },
            )
            WorkSection(state = state, busyReason = busyReason, onFilter = viewModel::setFilter)
            InsightsSection(stats = state.stats, onSearch = { query -> onOpen(everything, query) })
        }
    }
    DialogHost(controller = viewModel.dialogs)
    ReviewDialog(controller = viewModel.review)
}
