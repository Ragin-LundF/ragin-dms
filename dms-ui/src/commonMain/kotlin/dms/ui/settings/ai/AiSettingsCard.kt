package dms.ui.settings.ai

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.AutoAwesome
import androidx.compose.material.icons.rounded.NetworkCheck
import androidx.compose.material.icons.rounded.Save
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.application.port.state.model.AiSettings
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButton
import dms.ui.components.model.ButtonStyle
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.CardTitle
import dms.ui.settings.ai.model.AiDraft
import dms.ui.settings.model.AiConnection
import dms.ui.theme.Spacing

/** Where the AI runs: the server address, the model, and a test that lists the server's models. */
@Composable
fun AiSettingsCard(
    settings: AiSettings,
    connection: AiConnection,
    busyReason: String?,
    onSave: (AiSettings) -> Unit,
    onTest: (AiSettings) -> Unit,
) {
    var draft by remember(settings) { mutableStateOf(AiDraft.of(settings = settings)) }
    val typed = draft.toSettings()
    AppCard(modifier = Modifier.widthIn(max = 920.dp)) {
        CardTitle(
            title = "AI",
            icon = Icons.Rounded.AutoAwesome,
            tint = MaterialTheme.colorScheme.primary,
            subtitle = "An OpenAI-compatible server, such as LM Studio's local server. It only suggests; you decide",
        )
        Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.medium)) {
            AiServerFields(draft = draft, onChange = { draft = it })
            if (connection.models.isNotEmpty()) {
                AiModelChips(
                    models = connection.models,
                    selected = draft.model,
                    onSelect = { draft = draft.copy(model = it) },
                )
            }
            AiSwitches(draft = draft, onChange = { draft = it })
            AiConnectionLine(connection = connection)
            ActionButtons(
                buttons = listOf(
                    ActionButton(
                        label = "Test connection",
                        icon = Icons.Rounded.NetworkCheck,
                        disabledReason = if (typed == null) "Enter a valid address and timeout" else null,
                        onClick = { typed?.let(onTest) },
                    ),
                    ActionButton(
                        label = "Save",
                        icon = Icons.Rounded.Save,
                        style = ButtonStyle.TONAL,
                        disabledReason = when {
                            typed == null -> "Enter a valid address and timeout"
                            typed == settings -> "Nothing changed"
                            else -> null
                        },
                        onClick = { typed?.let(onSave) },
                    ),
                ),
                enabled = busyReason == null,
                busyReason = busyReason,
            )
        }
    }
}
