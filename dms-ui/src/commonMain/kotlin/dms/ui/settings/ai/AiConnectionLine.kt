package dms.ui.settings.ai

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.CheckCircle
import androidx.compose.material.icons.rounded.ErrorOutline
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.ui.settings.model.AiConnection
import dms.ui.theme.Spacing

/** What the last connection test found, with an icon so the result does not rely on color. */
@Composable
fun AiConnectionLine(connection: AiConnection) {
    val message = if (connection.checking) "Asking the server for its models…" else connection.message ?: return
    val color = if (connection.isError) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurfaceVariant
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.small),
    ) {
        when {
            connection.checking -> CircularProgressIndicator(
                modifier = Modifier.size(size = 16.dp),
                strokeWidth = 2.dp,
            )

            connection.isError -> Icon(
                imageVector = Icons.Rounded.ErrorOutline,
                contentDescription = null,
                tint = color,
            )

            else -> Icon(
                imageVector = Icons.Rounded.CheckCircle,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.primary,
            )
        }
        Text(text = message, style = MaterialTheme.typography.bodySmall, color = color)
    }
}
