package dms.ui.library.selection

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.DriveFileMove
import androidx.compose.material.icons.rounded.AutoAwesome
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import dms.ui.library.LibraryViewModel
import dms.ui.library.model.LibraryUiStateModel
import dms.ui.session.OutcomeMessages
import dms.ui.theme.Spacing

/**
 * The floating bar while documents are ticked: "N selected · Category · Tag · Move · Tag with AI ·
 * Trash… · ×". The library's actions wait for a busy library, "Tag with AI" for a busy AI.
 */
@Composable
fun SelectionBar(state: LibraryUiStateModel, viewModel: LibraryViewModel, modifier: Modifier = Modifier) {
    val busyReason = state.busy?.let { OutcomeMessages.waitFor(operation = it) }
    val aiBusyReason = state.aiBusy?.let { OutcomeMessages.waitFor(operation = it) }
    val colors = MaterialTheme.colorScheme
    val bulk = viewModel.bulk
    Surface(
        modifier = modifier,
        shape = MaterialTheme.shapes.large,
        color = colors.inverseSurface,
        contentColor = colors.inverseOnSurface,
        shadowElevation = Spacing.small,
    ) {
        Row(
            modifier = Modifier.padding(
                start = Spacing.large,
                end = Spacing.xsmall,
                top = Spacing.xsmall,
                bottom = Spacing.xsmall,
            ),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.xsmall),
        ) {
            Text(
                text = "${state.selectedDocuments.size} selected",
                fontWeight = FontWeight.SemiBold,
                maxLines = 1,
                softWrap = false,
                modifier = Modifier.padding(end = Spacing.small),
            )
            CategoryMenuButton(state = state, disabledReason = busyReason, bulk = bulk)
            TagMenuButton(state = state, disabledReason = busyReason, bulk = bulk)
            BarButton(
                label = "Move",
                icon = Icons.AutoMirrored.Rounded.DriveFileMove,
                disabledReason = busyReason,
                onClick = bulk::move,
            )
            BarButton(
                label = "Tag with AI",
                icon = Icons.Rounded.AutoAwesome,
                disabledReason = aiBusyReason,
                ai = true,
                onClick = bulk::tagWithAi,
            )
            BarButton(
                label = "Trash…",
                icon = Icons.Rounded.Delete,
                disabledReason = busyReason,
                onClick = bulk::trash,
            )
            IconButton(onClick = viewModel.documentSelection::clear) {
                Icon(imageVector = Icons.Rounded.Close, contentDescription = "Clear the selection")
            }
        }
    }
}
