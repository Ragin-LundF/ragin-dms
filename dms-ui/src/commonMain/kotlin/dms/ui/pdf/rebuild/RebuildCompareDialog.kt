package dms.ui.pdf.rebuild

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.KeyboardArrowLeft
import androidx.compose.material.icons.automirrored.rounded.KeyboardArrowRight
import androidx.compose.material.icons.rounded.AutoFixHigh
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.DialogProperties
import dms.application.pdf.rebuild.model.RebuildChoice
import dms.domain.rebuild.RegionKind
import dms.domain.rebuild.model.LayoutBoxModel
import dms.domain.rebuild.model.RebuildRegionModel
import dms.ui.components.format.FileSizes
import dms.ui.pdf.model.CompareView
import dms.ui.pdf.model.RebuildCompareStateModel
import dms.ui.theme.ChartColors
import dms.ui.theme.Spacing

/** The scan and its rebuild, compared page by page, with the choice of which to keep. */
@Composable
fun RebuildCompareDialog(controller: RebuildController) {
    val open by controller.compare.collectAsState()
    val state = open ?: return
    RebuildCompare(
        state = state,
        onChoose = controller::choose,
        onPage = controller::showPage,
        onFlip = controller::flip,
        onSideBySide = controller::sideBySide,
        onOverlay = controller::toggleOverlay,
    )
}

@Composable
fun RebuildCompare(
    state: RebuildCompareStateModel,
    onChoose: (RebuildChoice) -> Unit,
    onPage: (Int) -> Unit,
    onFlip: () -> Unit,
    onSideBySide: () -> Unit,
    onOverlay: () -> Unit,
) {
    // Only the three buttons close it: a stray click outside or Esc must not throw the rebuild away.
    AlertDialog(
        onDismissRequest = {},
        properties = DialogProperties(
            usePlatformDefaultWidth = false,
            dismissOnClickOutside = false,
            dismissOnBackPress = false,
        ),
        modifier = Modifier.fillMaxWidth(fraction = 0.9f).widthIn(max = 1000.dp),
        icon = { Icon(imageVector = Icons.Rounded.AutoFixHigh, contentDescription = null) },
        title = { Text(text = "Compare with the rebuilt PDF") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
                Summary(state = state)
                Controls(
                    state = state,
                    onPage = onPage,
                    onFlip = onFlip,
                    onSideBySide = onSideBySide,
                    onOverlay = onOverlay,
                )
                Panes(state = state)
            }
        },
        confirmButton = {
            Row(horizontalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
                TextButton(onClick = { onChoose(RebuildChoice.KEEP_ORIGINAL) }) { Text(text = "Keep original") }
                OutlinedButton(onClick = { onChoose(RebuildChoice.KEEP_BOTH) }) { Text(text = "Keep both") }
                Button(onClick = { onChoose(RebuildChoice.REPLACE_WITH_REBUILD) }) { Text(text = "Use rebuilt PDF") }
            }
        },
    )
}

@Composable
private fun Summary(state: RebuildCompareStateModel) {
    val page = state.rebuilt.pages[state.page]
    val sizes = "Original ${FileSizes.format(bytes = state.originalSize)} · " +
        "rebuilt ${FileSizes.format(bytes = state.rebuilt.size)}"
    Text(text = "${page.linesAsText} of ${page.linesTotal} lines on this page are real text · $sizes")
    val kept = page.keptReasons.entries.sortedByDescending { it.value }
    if (kept.isNotEmpty()) {
        Text(
            text = "Kept as images: " + kept.joinToString(separator = "; ") { (reason, count) ->
                "$count × ${RebuildTexts.reason(reason = reason)}"
            },
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

@Composable
private fun Controls(
    state: RebuildCompareStateModel,
    onPage: (Int) -> Unit,
    onFlip: () -> Unit,
    onSideBySide: () -> Unit,
    onOverlay: () -> Unit,
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
    ) {
        IconButton(onClick = { onPage(state.page - 1) }, enabled = state.page > 0) {
            Icon(imageVector = Icons.AutoMirrored.Rounded.KeyboardArrowLeft, contentDescription = "Previous page")
        }
        Text(text = "Page ${state.page + 1} of ${state.pages}")
        IconButton(onClick = { onPage(state.page + 1) }, enabled = state.page < state.pages - 1) {
            Icon(imageVector = Icons.AutoMirrored.Rounded.KeyboardArrowRight, contentDescription = "Next page")
        }
        OutlinedButton(onClick = onFlip) { Text(text = "Flip") }
        if (state.view != CompareView.SIDE_BY_SIDE) {
            TextButton(onClick = onSideBySide) { Text(text = "Side by side") }
        }
        Checkbox(checked = state.overlay, onCheckedChange = { onOverlay() })
        Text(text = "Mark text, kept lines and images")
    }
    if (state.overlay) {
        Legend()
    }
}

@Composable
private fun Legend() {
    Row(horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium)) {
        RegionKind.entries.forEach { kind ->
            Text(
                text = "■ ${labelOf(kind = kind)}",
                color = colorOf(kind = kind),
                style = MaterialTheme.typography.bodySmall,
            )
        }
    }
}

@Composable
private fun Panes(state: RebuildCompareStateModel) {
    Row(
        modifier = Modifier.fillMaxWidth().height(height = 620.dp),
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
    ) {
        if (state.view != CompareView.REBUILT) {
            Pane(
                label = "Original",
                image = state.original,
                regions = emptyList(),
                modifier = Modifier.weight(weight = 1f),
            )
        }
        if (state.view != CompareView.ORIGINAL) {
            val regions = if (state.overlay) state.rebuilt.pages[state.page].regions else emptyList()
            Pane(
                label = "Rebuilt",
                image = state.rebuiltImage,
                regions = regions,
                modifier = Modifier.weight(weight = 1f),
            )
        }
    }
}

@Composable
private fun Pane(label: String, image: ImageBitmap?, regions: List<RebuildRegionModel>, modifier: Modifier) {
    Column(modifier = modifier, horizontalAlignment = Alignment.CenterHorizontally) {
        Text(text = label, style = MaterialTheme.typography.labelLarge)
        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            if (image == null) {
                CircularProgressIndicator()
                return@Box
            }
            Box(modifier = Modifier.aspectRatio(ratio = image.width.toFloat() / image.height)) {
                Image(
                    bitmap = image,
                    contentDescription = "$label page",
                    contentScale = ContentScale.Fit,
                    modifier = Modifier.fillMaxSize(),
                )
                Overlay(regions = regions)
            }
        }
    }
}

@Composable
private fun Overlay(regions: List<RebuildRegionModel>) {
    if (regions.isEmpty()) return
    val colors = RegionKind.entries.associateWith { colorOf(kind = it) }
    Canvas(modifier = Modifier.fillMaxSize()) {
        regions.forEach { region ->
            val box = region.box
            val scaleX = size.width / LayoutBoxModel.SCALE.toFloat()
            val scaleY = size.height / LayoutBoxModel.SCALE.toFloat()
            drawRect(
                color = colors.getValue(key = region.kind),
                topLeft = Offset(x = box.left.toFloat() * scaleX, y = box.top.toFloat() * scaleY),
                size = Size(
                    width = (box.right - box.left).toFloat() * scaleX,
                    height = (box.bottom - box.top).toFloat() * scaleY,
                ),
                style = Stroke(width = 1.5f),
            )
        }
    }
}

/** Blue, orange and green: the first three chart colours, safe for colour blindness. */
@Composable
private fun colorOf(kind: RegionKind): Color {
    return when (kind) {
        RegionKind.IMAGE -> ChartColors.slot(index = 0)
        RegionKind.KEPT_LINE -> ChartColors.slot(index = 1)
        RegionKind.TEXT -> ChartColors.slot(index = 2)
    }
}

private fun labelOf(kind: RegionKind): String {
    return when (kind) {
        RegionKind.TEXT -> "real text"
        RegionKind.KEPT_LINE -> "line kept as image"
        RegionKind.IMAGE -> "picture of the scan"
    }
}
