package dms.ui.components.surface

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import dms.domain.document.DocumentKind

/** The coloured tile that tells PDFs, images and Office files apart at a glance. */
@Composable
fun KindBadge(kind: DocumentKind, modifier: Modifier = Modifier, size: Dp = 36.dp) {
    IconBadge(
        icon = KindStyles.icon(kind = kind),
        tint = KindStyles.color(kind = kind),
        modifier = modifier,
        size = size,
    )
}
