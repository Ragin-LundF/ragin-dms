package dms.ui.library

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.library.filter.FilterRow
import dms.ui.library.model.LibraryUiStateModel
import dms.ui.library.search.LibrarySearch
import dms.ui.theme.Spacing

/**
 * The search over the shown documents and below it the filter row. The selection's title and "Add
 * files" sit in the window toolbar.
 */
@Composable
fun LibraryToolbar(state: LibraryUiStateModel, busyReason: String?, viewModel: LibraryViewModel) {
    Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.medium)) {
        LibrarySearch(
            query = state.query,
            search = state.contentSearch,
            onSearch = viewModel::search,
            modifier = Modifier.fillMaxWidth(),
        )
        FilterRow(
            state = state,
            busyReason = busyReason,
            onRemove = viewModel::removeFilter,
            onYear = viewModel::filterYear,
            onContentSearch = viewModel.contentSearch::enable,
            onLayout = viewModel::setLayout,
            onSaveSmartFolder = viewModel.smartFolders::saveCurrent,
            onNewFolder = viewModel::newFolder,
            onExport = { viewModel.export.openCurrentView(title = state.title) },
        )
        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
    }
}
