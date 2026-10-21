package dms.ui.dashboard.stats.model

import dms.domain.document.DocumentId
import dms.domain.document.DocumentKind
import java.time.Instant

/** A recently added document as the dashboard lists it. */
data class RecentDocument(
    val id: DocumentId,
    val title: String,
    val folder: String,
    val kind: DocumentKind,
    val categoryName: String?,
    val addedAt: Instant,
)
