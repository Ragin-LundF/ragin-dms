package dms.ui.library.sidebar

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ViewSidebar
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import dms.ui.components.ReasonTooltip
import dms.ui.library.LibraryViewModel
import dms.ui.library.model.LibraryUiStateModel
import dms.ui.library.sidebar.item.SidebarItem
import dms.ui.shell.LocalCommandKey
import dms.ui.theme.Spacing

/**
 * The library's views: the "Library" section and the saved views. Trashed documents are in the system
 * trash, so the sidebar has no Trash entry. The panel button at the top folds the sidebar into an icon
 * rail ([folded]) and back.
 */
@Composable
fun LibrarySidebar(
    state: LibraryUiStateModel,
    folded: Boolean,
    busyReason: String?,
    viewModel: LibraryViewModel,
    modifier: Modifier = Modifier,
) {
    Surface(modifier = modifier, color = MaterialTheme.colorScheme.surfaceContainer) {
        Column(
            modifier = Modifier.fillMaxHeight().padding(horizontal = Spacing.small, vertical = Spacing.small),
            verticalArrangement = Arrangement.spacedBy(space = Spacing.hairline),
        ) {
            PanelButton(folded = folded, onClick = viewModel.sidebarWidth::toggleFolded)
            Column(
                modifier = Modifier.weight(weight = 1f).verticalScroll(state = rememberScrollState()),
                verticalArrangement = Arrangement.spacedBy(space = Spacing.hairline),
            ) {
                ViewEntries(state = state, folded = folded, viewModel = viewModel)
                SmartFolderEntries(state = state, folded = folded, busyReason = busyReason, viewModel = viewModel)
            }
        }
    }
}

@Composable
private fun PanelButton(folded: Boolean, onClick: () -> Unit) {
    val label = "${if (folded) "Expand" else "Collapse"} sidebar (${LocalCommandKey.current}\\)"
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = if (folded) Arrangement.Center else Arrangement.End,
    ) {
        ReasonTooltip(reason = label, beside = folded) {
            IconButton(onClick = onClick) {
                Icon(
                    imageVector = Icons.AutoMirrored.Rounded.ViewSidebar,
                    contentDescription = label,
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}
