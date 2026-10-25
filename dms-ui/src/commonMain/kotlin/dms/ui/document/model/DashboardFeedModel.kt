package dms.ui.document.model

import dms.application.port.state.model.ActivityEntryModel
import dms.domain.classification.model.ClassificationSuggestionModel
import dms.domain.document.model.DocumentIdModel

/**
 * What the dashboard knows about documents: the [suggestions] that wait for a review, by document,
 * and the [activity] of this device, newest first.
 */
data class DashboardFeedModel(
    val suggestions: Map<DocumentIdModel, ClassificationSuggestionModel> = emptyMap(),
    val activity: List<ActivityEntryModel> = emptyList(),
)
