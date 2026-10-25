package dms.ui.library.filter

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Archive
import androidx.compose.material.icons.rounded.GridView
import androidx.compose.material.icons.rounded.TableRows
import androidx.compose.material.icons.rounded.FindInPage
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import dms.ui.components.ActionMenu
import dms.ui.components.model.ActionButtonModel
import dms.ui.library.LibraryViewModel
import dms.ui.library.documents.model.GroupKey
import dms.ui.library.model.DocumentLayout
import dms.ui.library.model.LibraryUiStateModel
import dms.ui.theme.Spacing

/**
 * The right end of the chip row: group by month, category or sender (in the grid), the grid/list switch,
 * and a ⋯ menu with Export… and whether the search reads the documents' text.
 */
@Composable
fun ViewControls(state: LibraryUiStateModel, busyReason: String?, viewModel: LibraryViewModel) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
    ) {
        Segments {
            GroupKey.entries.forEach { key ->
                Segment(selected = state.groupBy == key, label = key.label, icon = null) {
                    viewModel.groupBy(key = key)
                }
            }
        }
        Segments {
            Segment(
                selected = state.layout == DocumentLayout.GRID,
                label = null,
                icon = Icons.Rounded.GridView,
                description = "Grid",
            ) {
                viewModel.setLayout(layout = DocumentLayout.GRID)
            }
            Segment(
                selected = state.layout == DocumentLayout.LIST,
                label = null,
                icon = Icons.Rounded.TableRows,
                description = "List",
            ) {
                viewModel.setLayout(layout = DocumentLayout.LIST)
            }
        }
        val searchesText = state.contentSearch.enabled
        ActionMenu(
            actions = listOf(
                ActionButtonModel(
                    label = "Export…",
                    icon = Icons.Rounded.Archive,
                    disabledReason = state.queryProblem ?: "Nothing to export".takeIf { state.documents.isEmpty() },
                    onClick = { viewModel.export.openCurrentView(title = state.title) },
                ),
                ActionButtonModel(
                    label = if (searchesText) "Search names only" else "Search the documents' text too",
                    icon = Icons.Rounded.FindInPage,
                    onClick = { viewModel.contentSearch.enable(on = !searchesText) },
                ),
            ),
            contentDescription = "More library actions",
            enabled = busyReason == null,
            busyReason = busyReason,
        )
    }
}

@Composable
private fun Segments(content: @Composable () -> Unit) {
    Surface(shape = MaterialTheme.shapes.medium, color = MaterialTheme.colorScheme.surfaceContainer) {
        Row(modifier = Modifier.padding(all = Spacing.hairline * 2)) { content() }
    }
}

@Composable
private fun Segment(
    selected: Boolean,
    label: String?,
    icon: ImageVector?,
    description: String? = null,
    onClick: () -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    Row(
        modifier = Modifier
            .clip(shape = MaterialTheme.shapes.small)
            .clickable(role = Role.Tab, onClick = onClick)
            .semantics {
                this.selected = selected
                description?.let { contentDescription = it }
            },
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Surface(shape = MaterialTheme.shapes.small, color = if (selected) colors.surface else Color.Transparent) {
            Row(
                modifier = Modifier.padding(horizontal = Spacing.small + Spacing.xsmall / 2, vertical = Spacing.xsmall),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                icon?.let {
                    Icon(
                        imageVector = it,
                        contentDescription = null,
                        tint = if (selected) colors.onSurface else colors.onSurfaceVariant,
                        modifier = Modifier.size(size = Spacing.large),
                    )
                }
                label?.let {
                    Text(
                        text = it,
                        style = MaterialTheme.typography.labelMedium,
                        color = if (selected) colors.onSurface else colors.onSurfaceVariant,
                        maxLines = 1,
                        softWrap = false,
                    )
                }
            }
        }
    }
}
