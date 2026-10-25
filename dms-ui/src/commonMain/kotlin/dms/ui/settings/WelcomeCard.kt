package dms.ui.settings

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.FolderSpecial
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ButtonStyle
import dms.ui.components.surface.AppCard
import dms.ui.theme.Spacing

/** The first thing a new user sees: what the primary folder is, and choosing it. */
@Composable
fun WelcomeCard(busyReason: String?, onChoose: () -> Unit) {
    AppCard(modifier = Modifier.widthIn(max = 920.dp), contentPadding = Spacing.xxlarge) {
        Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.large)) {
            Text(text = "Welcome", style = MaterialTheme.typography.headlineLarge)
            Text(
                text = "Choose the folder that holds your documents. It becomes the primary folder: every change " +
                    "is written there first, then to every sync folder. Your files stay readable files on disk, " +
                    "with their original names.",
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            ActionButtons(
                buttons = listOf(
                    ActionButtonModel(
                        label = "Choose primary folder…",
                        icon = Icons.Rounded.FolderSpecial,
                        style = ButtonStyle.FILLED,
                        onClick = onChoose,
                    ),
                ),
                enabled = busyReason == null,
                busyReason = busyReason,
            )
        }
    }
}
