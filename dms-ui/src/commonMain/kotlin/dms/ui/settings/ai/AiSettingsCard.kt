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
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dms.application.port.state.model.AiProvider
import dms.application.port.state.model.AiSettingsModel
import dms.ui.components.ActionButtons
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.model.ButtonStyle
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.CardTitle
import dms.ui.settings.ai.model.AiDraftModel
import dms.ui.settings.model.AiConnectionModel
import dms.ui.theme.Spacing

/** Where the AI runs, Local AI or Anthropic, its model, and a test that lists the provider's models. */
@Composable
fun AiSettingsCard(
    settings: AiSettingsModel,
    connection: AiConnectionModel,
    busyReason: String?,
    onSave: (AiSettingsModel) -> Unit,
    onTest: (AiSettingsModel) -> Unit,
) {
    var draft by rememberSaveable(settings) { mutableStateOf(AiDraftModel.of(settings = settings)) }
    val typed = draft.toSettings()
    // The listed models belong to the provider that was tested; another provider's names would not work.
    var testedProvider by rememberSaveable { mutableStateOf<AiProvider?>(value = null) }
    AppCard(modifier = Modifier.widthIn(max = 920.dp)) {
        CardTitle(
            title = "AI",
            icon = Icons.Rounded.AutoAwesome,
            tint = MaterialTheme.colorScheme.primary,
            subtitle = "Local server or Anthropic. It only suggests; you decide",
        )
        Column(verticalArrangement = Arrangement.spacedBy(space = Spacing.medium)) {
            SettingSwitch(
                title = "Use AI",
                text = "Shows the AI features: suggestions, tags, summaries, questions and converting scans to " +
                    "real PDFs. When off, the app shows none of them and sends nothing anywhere.",
                checked = draft.enabled,
                onCheckedChange = { draft = draft.copy(enabled = it) },
            )
            if (draft.enabled) {
                // The general switches come first, so they keep their place when the provider fields change.
                AiSwitches(draft = draft, onChange = { draft = it })
                AiServerFields(draft = draft, onChange = { draft = it })
                if (connection.models.isNotEmpty() && testedProvider == draft.provider) {
                    ProviderModelChips(models = connection.models, draft = draft, onChange = { draft = it })
                }
                AiConnectionLine(connection = connection)
            }
            ActionButtons(
                buttons = listOfNotNull(
                    ActionButtonModel(
                        label = "Test connection",
                        icon = Icons.Rounded.NetworkCheck,
                        disabledReason = draft.problem(),
                        onClick = {
                            typed?.let { tested ->
                                testedProvider = tested.provider
                                onTest(tested)
                            }
                        },
                    ).takeIf { draft.enabled },
                    ActionButtonModel(
                        label = "Save",
                        icon = Icons.Rounded.Save,
                        style = ButtonStyle.TONAL,
                        disabledReason = when {
                            typed == null -> draft.problem()
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

/** The tested provider's models; a pick fills the model field of the chosen provider. */
@Composable
private fun ProviderModelChips(models: List<String>, draft: AiDraftModel, onChange: (AiDraftModel) -> Unit) {
    val anthropic = draft.provider == AiProvider.ANTHROPIC
    AiModelChips(
        models = models,
        selected = if (anthropic) draft.anthropicModel else draft.model,
        firstLabel = if (anthropic) "Default model" else "First loaded model",
        onSelect = { model ->
            onChange(if (anthropic) draft.copy(anthropicModel = model) else draft.copy(model = model))
        },
    )
}
