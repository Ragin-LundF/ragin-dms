package dms.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable

/** Material 3 with the app's colours, type and shapes, in light or dark following the system setting. */
@Composable
fun AppTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = if (isSystemInDarkTheme()) AppColors.dark else AppColors.light,
        typography = AppTypography.typography,
        shapes = AppShapes.shapes,
        content = content,
    )
}
