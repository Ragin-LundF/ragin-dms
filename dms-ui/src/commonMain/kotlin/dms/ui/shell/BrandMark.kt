package dms.ui.shell

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Inventory2
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import dms.ui.theme.AppColors
import dms.ui.theme.LocalAppLogo
import dms.ui.theme.Spacing

/** The logo tile in the brand gradient and the app's name. */
@Composable
fun BrandMark() {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.medium),
    ) {
        val logo = LocalAppLogo.current
        if (logo != null) {
            Image(painter = logo, contentDescription = "Ragin-DMS logo", modifier = Modifier.size(size = 44.dp))
        } else {
            Box(
                modifier = Modifier
                    .size(size = 40.dp)
                    .clip(shape = MaterialTheme.shapes.medium)
                    .background(brush = Brush.linearGradient(colors = AppColors.brandGradient)),
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    imageVector = Icons.Rounded.Inventory2,
                    contentDescription = null,
                    tint = Color.White,
                    modifier = Modifier.size(size = 22.dp),
                )
            }
        }
        Column {
            Text(text = "Ragin-DMS", style = MaterialTheme.typography.titleMedium)
            Text(
                text = "Your documents",
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}
