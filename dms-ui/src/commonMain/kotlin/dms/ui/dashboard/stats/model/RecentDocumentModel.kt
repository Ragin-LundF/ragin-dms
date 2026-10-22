package dms.ui.dashboard.stats.model

import dms.domain.document.DocumentKind
import dms.domain.document.model.DocumentIdModel
import java.time.Instant

/** A recently added document as the dashboard lists it. */
data class RecentDocumentModel(
    val id: DocumentIdModel,
    val title: String,
    val folder: String,
    val kind: DocumentKind,
    val categoryName: String?,
    val addedAt: Instant,
)
