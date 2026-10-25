package dms.ui.dashboard.widgets

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import dms.ui.dashboard.model.TaskFileModel
import dms.ui.preview.FileThumbnail
import dms.ui.preview.ThumbnailLoader
import dms.ui.theme.Spacing

/**
 * The small page picture at the start of a dashboard row: [file] as a thumbnail, or [fallback] for
 * rows without a file and files without a picture, at most as wide as [Spacing.xxlarge]. The box,
 * a little taller than wide like a page, keeps every row the same size.
 */
@Composable
fun RowThumbnail(
    file: TaskFileModel?,
    loader: ThumbnailLoader,
    onClick: () -> Unit,
    fallback: @Composable () -> Unit,
) {
    Box(
        modifier = Modifier.size(width = Spacing.xxlarge, height = Spacing.xxlarge + Spacing.small),
        contentAlignment = Alignment.Center,
    ) {
        if (file == null) {
            fallback()
        } else {
            FileThumbnail(
                folderId = file.folderId,
                path = file.path,
                loader = loader,
                onClick = onClick,
                fallback = fallback,
            )
        }
    }
}
