package dms.ui.library.documents.grid

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import dms.domain.document.DocumentKind
import dms.domain.field.FieldValues
import dms.domain.library.model.LibraryFolderIdModel
import dms.ui.components.format.DateTexts
import dms.ui.components.surface.KindStyles
import dms.ui.library.model.DocumentRowModel
import dms.ui.preview.PageThumbnail
import dms.ui.preview.ThumbnailLoader
import dms.ui.theme.AppColors
import dms.ui.theme.Spacing

/**
 * The first page of a document as a sheet of paper, from [loader]; until it is there, and for files
 * without one, a sheet in the colour of the document's kind. The due date and the amount sit on the page.
 */
@Composable
fun PagePreview(
    row: DocumentRowModel,
    folderId: LibraryFolderIdModel?,
    loader: ThumbnailLoader,
    modifier: Modifier = Modifier,
) {
    val colors = MaterialTheme.colorScheme
    val shape = MaterialTheme.shapes.small
    val document = row.document
    Box(
        modifier = modifier
            .clip(shape = shape)
            .background(color = colors.surface)
            .border(width = Spacing.hairline, color = colors.outlineVariant, shape = shape),
    ) {
        if (folderId == null) {
            BlankPage(kind = document.kind)
        } else {
            PageThumbnail(
                folderId = folderId,
                path = document.path,
                loader = loader,
                modifier = Modifier.fillMaxSize(),
            ) {
                BlankPage(kind = document.kind)
            }
        }
        Column(
            modifier = Modifier.align(alignment = Alignment.BottomEnd).padding(all = Spacing.small),
            horizontalAlignment = Alignment.End,
            verticalArrangement = Arrangement.spacedBy(space = Spacing.xsmall),
        ) {
            row.dueDate?.let { PageLabel(text = "due ${DateTexts.absolute(date = it)}", warning = true) }
            row.amount?.let { PageLabel(text = FieldValues.format(value = it), warning = false) }
        }
    }
}

@Composable
private fun BlankPage(kind: DocumentKind) {
    val color = KindStyles.color(kind = kind)
    Column(modifier = Modifier.fillMaxSize(), horizontalAlignment = Alignment.CenterHorizontally) {
        Box(modifier = Modifier.fillMaxWidth().fillMaxHeight(fraction = BAND).background(color = color))
        Box(modifier = Modifier.weight(weight = 1f), contentAlignment = Alignment.Center) {
            Icon(
                imageVector = KindStyles.icon(kind = kind),
                contentDescription = KindStyles.label(kind = kind),
                tint = color.copy(alpha = 0.6f),
                modifier = Modifier.size(size = Spacing.xlarge),
            )
        }
    }
}

@Composable
private fun PageLabel(text: String, warning: Boolean) {
    val colors = MaterialTheme.colorScheme
    Surface(
        shape = MaterialTheme.shapes.extraSmall,
        color = colors.surface,
        contentColor = colors.onSurface,
        border = if (warning) null else BorderStroke(width = Spacing.hairline, color = colors.outlineVariant),
    ) {
        Text(
            text = text,
            style = MaterialTheme.typography.labelSmall,
            maxLines = 1,
            softWrap = false,
            modifier = Modifier
                .background(color = if (warning) AppColors.statusWarning.copy(alpha = WARNING_TINT) else colors.surface)
                .padding(horizontal = Spacing.xsmall + Spacing.hairline),
        )
    }
}

private const val BAND = 0.14f
private const val WARNING_TINT = 0.3f
