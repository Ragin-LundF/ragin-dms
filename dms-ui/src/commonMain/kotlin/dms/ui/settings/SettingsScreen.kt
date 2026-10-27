package dms.ui.settings

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.saveable.rememberSaveableStateHolder
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import dms.domain.settings.model.CatalogSettingsModel
import dms.ui.components.dialog.DialogHost
import dms.ui.components.layout.ScreenHeader
import dms.ui.session.OutcomeMessages
import dms.ui.settings.ai.AiSettingsCard
import dms.ui.settings.library.FieldsCard
import dms.ui.settings.library.InboxCard
import dms.ui.settings.library.RulesCard
import dms.ui.settings.model.SettingsSection
import dms.ui.settings.model.SettingsUiStateModel
import dms.ui.theme.Spacing

/**
 * The library folders of this device, how scans treat new files, and where the AI runs. A configured library
 * shows one section at a time, chosen in the [SettingsSectionList].
 */
@Composable
fun SettingsScreen(viewModel: SettingsViewModel) {
    val state by viewModel.state.collectAsState()
    val library by viewModel.library.state.collectAsState()
    val busyReason = state.busy?.let { OutcomeMessages.waitFor(operation = it) }
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(all = Spacing.screen),
        verticalArrangement = Arrangement.spacedBy(space = Spacing.xlarge),
    ) {
        ScreenHeader(title = "Settings", subtitle = "Where your library lives on this device")
        if (state.configured) {
            SectionedSettings(
                state = state,
                settings = library.settings,
                busyReason = busyReason,
                viewModel = viewModel,
            )
        } else {
            Column(
                modifier = Modifier.verticalScroll(state = rememberScrollState()),
                verticalArrangement = Arrangement.spacedBy(space = Spacing.xlarge),
            ) {
                WelcomeCard(busyReason = busyReason, onChoose = viewModel::choosePrimaryFolder)
                AiCard(state = state, busyReason = busyReason, viewModel = viewModel)
            }
        }
    }
    DialogHost(controller = viewModel.dialogs)
}

@Composable
private fun SectionedSettings(
    state: SettingsUiStateModel,
    settings: CatalogSettingsModel?,
    busyReason: String?,
    viewModel: SettingsViewModel,
) {
    val sections = availableSections(settingsLoaded = settings != null)
    var chosen by rememberSaveable { mutableStateOf(value = SettingsSection.FOLDERS) }
    val selected = chosen.takeIf { it in sections } ?: sections.first()
    // Keeps each hidden card's unsaved drafts (its rememberSaveable state) until the user comes back.
    // shortcut: a restored draft does not recheck its inputs, so settings saved elsewhere meanwhile (another
    // device) show the old draft until saved or reopened; key the drafts by their source if that matters.
    val drafts = rememberSaveableStateHolder()
    Row(horizontalArrangement = Arrangement.spacedBy(space = Spacing.xlarge)) {
        SettingsSectionList(sections = sections, selected = selected, onSelect = { chosen = it })
        drafts.SaveableStateProvider(key = selected) {
            Column(modifier = Modifier.weight(weight = 1f).verticalScroll(state = rememberScrollState())) {
                SectionCard(
                    section = selected,
                    state = state,
                    settings = settings,
                    busyReason = busyReason,
                    viewModel = viewModel,
                )
            }
        }
    }
}

/** The sections to list: those that edit the library's settings only once they are loaded. */
internal fun availableSections(settingsLoaded: Boolean): List<SettingsSection> {
    return SettingsSection.entries.filter { settingsLoaded || !it.needsLibrarySettings }
}

@Composable
private fun SectionCard(
    section: SettingsSection,
    state: SettingsUiStateModel,
    settings: CatalogSettingsModel?,
    busyReason: String?,
    viewModel: SettingsViewModel,
) {
    when (section) {
        SettingsSection.FOLDERS -> FoldersCard(rows = state.rows, busyReason = busyReason, viewModel = viewModel)
        SettingsSection.SCANNING -> ScanningCard(
            addNewFiles = state.addNewFilesWhileScanning,
            busyReason = busyReason,
            onAddNewFilesChange = viewModel::setAddNewFilesWhileScanning,
            onDetectDates = viewModel::detectDates,
        )
        SettingsSection.INBOX -> settings?.let {
            InboxCard(inbox = it.layout.inbox, busyReason = busyReason, controller = viewModel.library)
        }
        SettingsSection.FIELDS -> settings?.let {
            FieldsCard(settings = it, busyReason = busyReason, controller = viewModel.library)
        }
        SettingsSection.RULES -> settings?.let {
            RulesCard(
                settings = it,
                categoryName = viewModel.library::categoryName,
                busyReason = busyReason,
                controller = viewModel.library,
            )
        }
        SettingsSection.AI -> AiCard(state = state, busyReason = busyReason, viewModel = viewModel)
    }
}

@Composable
private fun AiCard(state: SettingsUiStateModel, busyReason: String?, viewModel: SettingsViewModel) {
    AiSettingsCard(
        settings = state.ai,
        connection = state.aiConnection,
        busyReason = busyReason,
        onSave = viewModel::saveAi,
        onTest = viewModel::testAi,
    )
}
