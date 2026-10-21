package dms.ui.components.surface

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Description
import androidx.compose.material.icons.rounded.Image
import androidx.compose.material.icons.rounded.InsertDriveFile
import androidx.compose.material.icons.rounded.PictureAsPdf
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import dms.domain.document.DocumentKind
import dms.ui.theme.ChartColors

/** Name, icon and colour of each kind of document; the colour is the same in every chart and badge. */
object KindStyles {

    fun label(kind: DocumentKind): String {
        return when (kind) {
            DocumentKind.PDF -> "PDF"
            DocumentKind.IMAGE -> "Image"
            DocumentKind.OFFICE -> "Office"
            DocumentKind.OTHER -> "Other"
        }
    }

    fun icon(kind: DocumentKind): ImageVector {
        return when (kind) {
            DocumentKind.PDF -> Icons.Rounded.PictureAsPdf
            DocumentKind.IMAGE -> Icons.Rounded.Image
            DocumentKind.OFFICE -> Icons.Rounded.Description
            DocumentKind.OTHER -> Icons.Rounded.InsertDriveFile
        }
    }

    @Composable
    fun color(kind: DocumentKind): Color {
        return when (kind) {
            DocumentKind.PDF -> ChartColors.slot(index = 1)
            DocumentKind.IMAGE -> ChartColors.slot(index = 2)
            DocumentKind.OFFICE -> ChartColors.slot(index = 0)
            DocumentKind.OTHER -> ChartColors.neutral
        }
    }
}
