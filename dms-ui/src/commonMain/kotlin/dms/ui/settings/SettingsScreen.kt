package dms.ui.settings

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import dms.ui.components.dialog.DialogHost
import dms.ui.components.layout.ScreenHeader
import dms.ui.session.OutcomeMessages
import dms.ui.settings.ai.AiSettingsCard
import dms.ui.settings.library.FieldsCard
import dms.ui.settings.library.InboxCard
import dms.ui.settings.library.RulesCard
import dms.ui.theme.Spacing

/** The library folders of this device, how scans treat new files, and where the AI runs. */
@Composable
fun SettingsScreen(viewModel: SettingsViewModel) {
    val state by viewModel.state.collectAsState()
    val library by viewModel.library.state.collectAsState()
    val busyReason = state.busy?.let { OutcomeMessages.waitFor(operation = it) }
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(state = rememberScrollState())
            .padding(all = Spacing.screen),
        verticalArrangement = Arrangement.spacedBy(space = Spacing.xlarge),
    ) {
        ScreenHeader(title = "Settings", subtitle = "Where your library lives on this device")
        if (!state.configured) {
            WelcomeCard(busyReason = busyReason, onChoose = viewModel::choosePrimaryFolder)
        } else {
            FoldersCard(rows = state.rows, busyReason = busyReason, viewModel = viewModel)
            ScanningCard(
                addNewFiles = state.addNewFilesWhileScanning,
                busyReason = busyReason,
                onAddNewFilesChange = viewModel::setAddNewFilesWhileScanning,
                onDetectDates = viewModel::detectDates,
            )
            library.settings?.let { settings ->
                InboxCard(inbox = settings.layout.inbox, busyReason = busyReason, controller = viewModel.library)
                FieldsCard(settings = settings, busyReason = busyReason, controller = viewModel.library)
                RulesCard(
                    settings = settings,
                    categoryName = viewModel.library::categoryName,
                    busyReason = busyReason,
                    controller = viewModel.library,
                )
            }
        }
        AiSettingsCard(
            settings = state.ai,
            connection = state.aiConnection,
            busyReason = busyReason,
            onSave = viewModel::saveAi,
            onTest = viewModel::testAi,
        )
    }
    DialogHost(controller = viewModel.dialogs)
}
