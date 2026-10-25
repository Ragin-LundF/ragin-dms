package dms.ui.dashboard.organize.model

import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.RelativePathModel

/**
 * One date ahead on the dashboard: when, what (a deadline field or the end of a retention), and of
 * which document, with its [path] for the thumbnail while the catalog still has it.
 */
data class UpcomingRowModel(
    val documentId: DocumentIdModel,
    val date: String,
    val what: String,
    val document: String,
    val path: RelativePathModel? = null,
)
