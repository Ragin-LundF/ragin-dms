package dms.ui.pdf

import androidx.compose.foundation.Image
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.RotateLeft
import androidx.compose.material.icons.automirrored.rounded.RotateRight
import androidx.compose.material.icons.rounded.ChevronLeft
import androidx.compose.material.icons.rounded.ChevronRight
import androidx.compose.material.icons.rounded.ContentCut
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.RestoreFromTrash
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.DialogProperties
import dms.ui.pdf.model.PageEditorStateModel
import dms.ui.pdf.model.PageItemModel
import dms.ui.theme.Spacing

/**
 * The pages of a PDF as a strip: turn, remove and move each page, and cut after a page to split the
 * document there. Nothing changes until the user applies.
 */
@Composable
fun PageEditorDialog(controller: PageEditorController) {
    val open by controller.state.collectAsState()
    val state = open ?: return
    AlertDialog(
        onDismissRequest = controller::cancel,
        properties = DialogProperties(usePlatformDefaultWidth = false),
        modifier = Modifier.width(width = 960.dp),
        title = { Text(text = "Pages of ${state.name}") },
        text = { PageStrip(state = state, controller = controller) },
        confirmButton = {
            val kept = state.pages.count { !it.deleted }
            val label = if (state.splits > 0) "Apply and split into ${state.splits + 1}" else "Apply"
            val changed = state.edited || state.splits > 0
            Button(onClick = controller::apply, enabled = kept > 0 && changed) { Text(text = label) }
        },
        dismissButton = { TextButton(onClick = controller::cancel) { Text(text = "Cancel") } },
    )
}

@Composable
private fun PageStrip(state: PageEditorStateModel, controller: PageEditorController) {
    Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.medium)) {
        if (state.loading) {
            CircularProgressIndicator()
            return@Column
        }
        LazyRow(
            horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
            modifier = Modifier.fillMaxWidth(),
        ) {
            itemsIndexed(items = state.pages, key = { _, page -> page.source }) { position, page ->
                PageCard(state = state, position = position, page = page, controller = controller)
            }
        }
        if (state.splits > 0) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Checkbox(checked = state.trashOriginal, onCheckedChange = controller::setTrashOriginal)
                Text(
                    text = "Move the original to the trash after splitting",
                    style = MaterialTheme.typography.bodyMedium,
                )
            }
        }
    }
}

@Composable
private fun PageCard(
    state: PageEditorStateModel,
    position: Int,
    page: PageItemModel,
    controller: PageEditorController,
) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier = Modifier.alpha(alpha = if (page.deleted) DELETED_ALPHA else 1f),
    ) {
        PagePicture(state = state, page = page)
        Text(text = "Page ${page.source + 1}", style = MaterialTheme.typography.labelMedium)
        PageButtons(position = position, page = page, controller = controller)
        if (page.splitAfter) {
            Text(
                text = "Split here",
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.primary,
            )
        }
    }
}

@Composable
private fun PagePicture(state: PageEditorStateModel, page: PageItemModel) {
    Box(
        contentAlignment = Alignment.Center,
        modifier = Modifier.size(
            width = 150.dp,
            height = 200.dp,
        ).border(width = 1.dp, color = MaterialTheme.colorScheme.outlineVariant),
    ) {
        val thumbnail = state.thumbnails[page.source]
        if (thumbnail == null) {
            Text(text = "${page.source + 1}", style = MaterialTheme.typography.headlineMedium)
        } else {
            Image(
                bitmap = thumbnail,
                contentDescription = "Page ${page.source + 1}",
                contentScale = ContentScale.Fit,
                modifier = Modifier.size(width = 140.dp, height = 190.dp).rotate(degrees = page.rotation.toFloat()),
            )
        }
    }
}

@Composable
private fun PageButtons(position: Int, page: PageItemModel, controller: PageEditorController) {
    Row {
        IconButton(onClick = { controller.move(position = position, by = -1) }) {
            Icon(imageVector = Icons.Rounded.ChevronLeft, contentDescription = "Move left")
        }
        IconButton(onClick = { controller.rotate(position = position, degrees = -QUARTER) }) {
            Icon(imageVector = Icons.AutoMirrored.Rounded.RotateLeft, contentDescription = "Turn left")
        }
        IconButton(onClick = { controller.rotate(position = position, degrees = QUARTER) }) {
            Icon(imageVector = Icons.AutoMirrored.Rounded.RotateRight, contentDescription = "Turn right")
        }
        IconButton(onClick = { controller.move(position = position, by = 1) }) {
            Icon(imageVector = Icons.Rounded.ChevronRight, contentDescription = "Move right")
        }
    }
    Row {
        IconButton(onClick = { controller.toggleDelete(position = position) }) {
            Icon(
                imageVector = if (page.deleted) Icons.Rounded.RestoreFromTrash else Icons.Rounded.Delete,
                contentDescription = if (page.deleted) "Keep page" else "Remove page",
            )
        }
        IconButton(onClick = { controller.toggleSplit(position = position) }) {
            Icon(
                imageVector = Icons.Rounded.ContentCut,
                contentDescription = "Split after this page",
                tint = if (page.splitAfter) {
                    MaterialTheme.colorScheme.primary
                } else {
                    MaterialTheme.colorScheme.onSurfaceVariant
                },
            )
        }
    }
}

private const val QUARTER = 90
private const val DELETED_ALPHA = 0.35f
