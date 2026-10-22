package dms.ui.settings.library

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.MoveToInbox
import androidx.compose.material.icons.rounded.Save
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.domain.settings.model.InboxSettingsModel
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ButtonStyle
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.CardTitle
import dms.ui.theme.Spacing

/**
 * The inbox: on or off, its folder inside the library (the same in every library folder), and how
 * much the AI does with what arrives. Edits apply when saved.
 */
@Composable
fun InboxCard(inbox: InboxSettingsModel, busyReason: String?, controller: LibrarySettingsController) {
    var enabled by remember(inbox) { mutableStateOf(value = inbox.enabled) }
    var path by remember(inbox) { mutableStateOf(value = inbox.path.value) }
    var aiAnalysis by remember(inbox) { mutableStateOf(value = inbox.aiAnalysis) }
    var aiTargetFolder by remember(inbox) { mutableStateOf(value = inbox.aiTargetFolder) }
    var autoAnalyze by remember(inbox) { mutableStateOf(value = inbox.autoAnalyze) }
    val problem = controller.inboxPathProblem(path = path)
    AppCard(modifier = Modifier.widthIn(max = 920.dp)) {
        CardTitle(
            title = "Inbox",
            icon = Icons.Rounded.MoveToInbox,
            tint = MaterialTheme.colorScheme.primary,
            subtitle = "A folder in your library where new documents arrive, on every device",
        )
        Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.medium)) {
            SwitchRow(text = "Use an inbox folder", checked = enabled) { enabled = it }
            OutlinedTextField(
                value = path,
                onValueChange = { path = it },
                label = { Text(text = "Folder inside the library") },
                singleLine = true,
                isError = problem != null,
                supportingText = { Text(text = problem ?: "Created in every library folder when you save") },
                enabled = enabled,
                modifier = Modifier.fillMaxWidth(),
            )
            SwitchRow(text = "Use AI for inbox documents", checked = aiAnalysis, enabled = enabled) { aiAnalysis = it }
            SwitchRow(
                text = "AI suggests the target folder",
                checked = aiTargetFolder,
                enabled = enabled && aiAnalysis,
            ) { aiTargetFolder = it }
            SwitchRow(
                text = "Analyze automatically when files arrive",
                checked = autoAnalyze,
                enabled = enabled && aiAnalysis,
            ) { autoAnalyze = it }
            ActionButtons(
                buttons = listOf(
                    ActionButtonModel(
                        label = "Save",
                        icon = Icons.Rounded.Save,
                        style = ButtonStyle.TONAL,
                        disabledReason = problem,
                    ) {
                        controller.saveInbox(
                            enabled = enabled,
                            path = path,
                            aiAnalysis = aiAnalysis,
                            aiTargetFolder = aiTargetFolder,
                            autoAnalyze = autoAnalyze,
                        )
                    },
                ),
                enabled = busyReason == null,
                busyReason = busyReason,
            )
        }
    }
}

@Composable
private fun SwitchRow(text: String, checked: Boolean, enabled: Boolean = true, onChange: (Boolean) -> Unit) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(space = Spacing.large),
    ) {
        Text(text = text, style = MaterialTheme.typography.bodyMedium, modifier = Modifier.weight(weight = 1f))
        Switch(checked = checked, onCheckedChange = onChange, enabled = enabled)
    }
}
