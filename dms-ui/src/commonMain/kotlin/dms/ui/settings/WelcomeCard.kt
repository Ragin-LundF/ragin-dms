package dms.ui.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.FolderSpecial
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import dms.ui.dashboard.widgets.HeroButton
import dms.ui.theme.AppColors
import dms.ui.theme.Spacing

/** The first thing a new user sees: what the primary folder is, and choosing it. */
@Composable
fun WelcomeCard(busyReason: String?, onChoose: () -> Unit) {
    Column(
        modifier = Modifier
            .widthIn(max = 920.dp)
            .clip(shape = MaterialTheme.shapes.extraLarge)
            .background(brush = Brush.linearGradient(colors = AppColors.brandGradient))
            .padding(all = Spacing.xxlarge),
        verticalArrangement = Arrangement.spacedBy(space = Spacing.large),
    ) {
        Text(text = "Welcome", style = MaterialTheme.typography.headlineLarge, color = Color.White)
        Text(
            text = "Choose the folder that holds your documents. It becomes the primary folder: every change " +
                "is written there first, then to every sync folder. Your files stay readable files on disk, " +
                "with their original names.",
            style = MaterialTheme.typography.bodyLarge,
            color = Color.White.copy(alpha = 0.9f),
        )
        HeroButton(
            label = "Choose primary folder…",
            icon = Icons.Rounded.FolderSpecial,
            busyReason = busyReason,
            onClick = onChoose,
        )
    }
}
