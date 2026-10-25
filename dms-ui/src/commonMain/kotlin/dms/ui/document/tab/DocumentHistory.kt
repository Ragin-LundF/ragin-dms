package dms.ui.document.tab

import dms.application.port.state.model.ActivityEntryModel
import dms.application.port.state.model.ActivityKind
import dms.domain.document.ClassificationSource
import dms.domain.document.model.ClassificationRecordModel
import dms.domain.document.model.DocumentModel
import dms.ui.dashboard.panel.ActivityTexts
import dms.ui.document.model.HistoryEntryModel

/**
 * What the app knows about a document's past, newest first: when it was added, who classified it
 * last, and the recent activity of this device on its path. The catalog keeps no full history.
 */
object DocumentHistory {

    fun of(document: DocumentModel, activity: List<ActivityEntryModel>): List<HistoryEntryModel> {
        val added = HistoryEntryModel(at = document.addedAt, text = "Added to the library")
        val classified = document.classification?.let { record ->
            HistoryEntryModel(at = record.confirmedAt, text = classificationText(record = record))
        }
        val recent = activity
            .filter { it.subject == document.path.value && it.kind != ActivityKind.DOCUMENT_ADDED }
            .map { HistoryEntryModel(at = it.at, text = ActivityTexts.verb(kind = it.kind)) }
        return (listOfNotNull(added, classified) + recent).sortedByDescending { it.at }
    }

    private fun classificationText(record: ClassificationRecordModel): String {
        return when (record.source) {
            ClassificationSource.AI -> "Classified as suggested by the AI (${record.model})"
            ClassificationSource.MANUAL -> "Classified by hand"
            ClassificationSource.RULE -> "Classified by a rule"
        }
    }
}
