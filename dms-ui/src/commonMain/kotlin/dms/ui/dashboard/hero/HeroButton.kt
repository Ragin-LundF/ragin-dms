package dms.ui.dashboard.hero

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import dms.ui.components.ReasonTooltip
import dms.ui.theme.Spacing

/** A translucent white button for the gradient header; disabled while the library is busy. */
@Composable
fun HeroButton(label: String, icon: ImageVector, busyReason: String?, onClick: () -> Unit) {
    ReasonTooltip(reason = busyReason) {
        Button(
            onClick = onClick,
            enabled = busyReason == null,
            colors = ButtonDefaults.buttonColors(
                containerColor = Color.White.copy(alpha = 0.16f),
                contentColor = Color.White,
                disabledContainerColor = Color.White.copy(alpha = 0.08f),
                disabledContentColor = Color.White.copy(alpha = 0.5f),
            ),
        ) {
            Icon(imageVector = icon, contentDescription = null, modifier = Modifier.size(size = Spacing.icon))
            Spacer(modifier = Modifier.width(width = Spacing.small))
            Text(text = label, maxLines = 1, softWrap = false)
        }
    }
}
