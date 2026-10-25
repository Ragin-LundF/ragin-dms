package dms.ui.dashboard.stats.model

import dms.domain.document.DocumentKind
import dms.domain.document.model.DocumentIdModel
import dms.domain.library.model.RelativePathModel

/**
 * A recently added document as the dashboard lists it: [added] in words ("2 days ago") and
 * [addedOn] as the exact date for its tooltip.
 */
data class RecentDocumentModel(
    val id: DocumentIdModel,
    val path: RelativePathModel,
    val title: String,
    val folder: String,
    val kind: DocumentKind,
    val categoryName: String?,
    val added: String,
    val addedOn: String,
)
