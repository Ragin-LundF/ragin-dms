package dms.ui.dashboard.stats.model

import dms.domain.document.DocumentKind

/** How many documents of one kind the library holds. */
data class KindCountModel(
    val kind: DocumentKind,
    val count: Int,
)
