package dms.ui.restructure

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.History
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import dms.ui.components.ActionButtons
import dms.ui.components.dialog.DialogHost
import dms.ui.components.layout.ScreenHeader
import dms.ui.components.model.ActionButtonModel
import dms.ui.components.surface.AppCard
import dms.ui.components.surface.CardTitle
import dms.ui.session.OutcomeMessages
import dms.ui.theme.Spacing

/** The restructure assistant: template, plan and the restructures that can be reversed. */
@Composable
fun RestructureScreen(viewModel: RestructureViewModel) {
    val state by viewModel.state.collectAsState()
    LaunchedEffect(key1 = viewModel) {
        viewModel.refresh()
    }
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(state = rememberScrollState())
            .padding(all = Spacing.screen),
        verticalArrangement = Arrangement.spacedBy(space = Spacing.xlarge),
    ) {
        ScreenHeader(title = "Restructure", subtitle = "Sort documents into folders by a template, after you approve")
        TemplateCard(state = state, viewModel = viewModel)
        if (state.planned) {
            PlanCard(state = state, viewModel = viewModel)
        }
        if (state.history.isNotEmpty()) {
            AppCard {
                CardTitle(
                    title = "Applied restructures",
                    icon = Icons.Rounded.History,
                    tint = MaterialTheme.colorScheme.tertiary,
                    subtitle = "Reverse puts back every document that is still where a restructure put it",
                )
                val busyReason = state.busy?.let { OutcomeMessages.waitFor(operation = it) }
                ActionButtons(
                    buttons = state.history.take(n = HISTORY_SHOWN).map { name ->
                        ActionButtonModel(label = "Reverse $name") { viewModel.reverse(name = name) }
                    },
                    enabled = busyReason == null,
                    busyReason = busyReason,
                )
            }
        }
    }
    DialogHost(controller = viewModel.dialogs)
}

private const val HISTORY_SHOWN = 5
