package dms.ui.dashboard.hero

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.AutoAwesome
import androidx.compose.material.icons.rounded.NewLabel
import androidx.compose.runtime.Composable
import dms.ui.dashboard.model.DashboardUiStateModel
import dms.ui.theme.Spacing

/** The AI's runs over the library, each with the number of documents it would read. */
@Composable
fun HeroAiButtons(state: DashboardUiStateModel, onTagAll: () -> Unit, onClassifyAll: () -> Unit) {
    val busyReason = state.aiBusy?.let { "Wait until \"$it\" finishes" }
    Row(horizontalArrangement = Arrangement.spacedBy(space = Spacing.small)) {
        HeroButton(
            label = "Tag with AI (${state.tagCandidates})",
            icon = Icons.Rounded.NewLabel,
            busyReason = busyReason ?: "Every document the AI can read has tags or a suggestion"
                .takeIf { state.tagCandidates == 0 },
            onClick = onTagAll,
        )
        HeroButton(
            label = "Classify with AI (${state.classifyCandidates})",
            icon = Icons.Rounded.AutoAwesome,
            busyReason = busyReason ?: "Every document the AI can read has a classification or a suggestion"
                .takeIf { state.classifyCandidates == 0 },
            onClick = onClassifyAll,
        )
    }
}
