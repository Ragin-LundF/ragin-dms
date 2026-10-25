package dms.ui.document.model

import java.time.Instant

/** Something that happened to a document [at] a time, in words. */
data class HistoryEntryModel(
    val at: Instant,
    val text: String,
)
