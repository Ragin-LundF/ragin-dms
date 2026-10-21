package dms.ui.components.surface

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import dms.ui.theme.Spacing

/** The card every dashboard and inspector block sits on: rounded, hairline border, soft shadow. */
@Composable
fun AppCard(
    modifier: Modifier = Modifier,
    onClick: (() -> Unit)? = null,
    contentPadding: Dp = Spacing.xlarge,
    content: @Composable ColumnScope.() -> Unit,
) {
    val shape = MaterialTheme.shapes.large
    val color = MaterialTheme.colorScheme.surface
    val border = BorderStroke(width = Spacing.hairline, color = MaterialTheme.colorScheme.outlineVariant)
    val inner: @Composable () -> Unit = {
        Column(modifier = Modifier.padding(all = contentPadding), content = content)
    }
    if (onClick == null) {
        Surface(
            modifier = modifier,
            shape = shape,
            color = color,
            border = border,
            shadowElevation = 1.dp,
            content = inner,
        )
    } else {
        Surface(
            onClick = onClick,
            modifier = modifier,
            shape = shape,
            color = color,
            border = border,
            shadowElevation = 1.dp,
            content = inner,
        )
    }
}
