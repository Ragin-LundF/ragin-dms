package dms.ui.dashboard.widgets

import dms.ui.dashboard.model.DashboardUiStateModel
import dms.ui.dashboard.widgets.model.AiBandModel
import dms.ui.session.OutcomeMessages

/**
 * What the AI band says and offers. It counts the documents the AI can read and would work on, so
 * the text matches what its buttons do; without any, there is no band.
 */
object AiBandContent {

    fun of(state: DashboardUiStateModel): AiBandModel? {
        val untagged = state.tagCandidates
        val unclassified = state.classifyCandidates
        if (untagged == 0 && unclassified == 0) {
            return null
        }
        return AiBandModel(
            headline = headline(untagged = untagged, unclassified = unclassified),
            classifyLabel = "Classify $unclassified".takeIf { unclassified > 0 },
            tagLabel = "Tag $untagged with AI".takeIf { untagged > 0 },
            busyReason = state.aiBusy?.let { OutcomeMessages.waitFor(operation = it) },
        )
    }

    private fun headline(untagged: Int, unclassified: Int): String {
        val noTags = "${has(count = untagged)} no tags yet"
        val noCategory = "${has(count = unclassified)} no category"
        return when {
            unclassified == 0 -> "${documents(count = untagged)} $noTags"
            untagged == 0 -> "${documents(count = unclassified)} $noCategory"
            else -> "${documents(count = untagged)} $noTags, $unclassified $noCategory"
        }
    }

    private fun documents(count: Int): String {
        return if (count == 1) "1 document" else "$count documents"
    }

    private fun has(count: Int): String {
        return if (count == 1) "has" else "have"
    }
}
