package dms.ui.dashboard.organize.model

import dms.domain.document.model.DocumentIdModel

/** One date ahead on the dashboard: when, what (a deadline field or the end of a retention), and of which document. */
data class UpcomingRowModel(
    val documentId: DocumentIdModel,
    val date: String,
    val what: String,
    val document: String,
)
